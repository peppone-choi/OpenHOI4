//! Allocation-free postcard shape preflight; sim performs semantic validation.
use crate::{Limits, Result};
struct Reader<'a> {
    bytes: &'a [u8],
    at: usize,
    limits: &'a Limits,
    charged: u64,
}
impl Reader<'_> {
    fn charge(&mut self, bytes: u64) -> Result<()> {
        self.charged = self
            .charged
            .checked_add(bytes)
            .ok_or("Limit: allocation budget")?;
        if self.charged > self.limits.allocation_budget_bytes {
            return Err("Limit: allocation budget".into());
        }
        Ok(())
    }
    fn entries(&mut self, cap: u64) -> Result<usize> {
        let length = self.length(cap)?;
        self.charge(
            (length as u64)
                .checked_mul(self.limits.entry_allocation_charge_bytes)
                .ok_or("Limit: allocation charge")?,
        )?;
        Ok(length)
    }
    fn byte(&mut self) -> Result<u8> {
        let b = *self.bytes.get(self.at).ok_or("Truncated: postcard field")?;
        self.at += 1;
        Ok(b)
    }
    fn number(&mut self, bits: u32) -> Result<u64> {
        let mut n = 0u64;
        for shift in (0..bits).step_by(7) {
            let b = self.byte()?;
            let value = u64::from(b & 127);
            if value > (u64::MAX >> shift) || (bits < 64 && value >= 1u64 << (bits - shift)) {
                return Err("Postcard: integer overflow".into());
            }
            n |= value << shift;
            if b & 128 == 0 {
                return Ok(n);
            }
        }
        Err("Postcard: varint overflow".into())
    }
    fn tag(&mut self, max: u64) -> Result<u64> {
        let tag = self.number(32)?;
        if tag > max {
            return Err("Postcard: enum tag".into());
        }
        Ok(tag)
    }
    fn boolean(&mut self) -> Result<bool> {
        match self.byte()? {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err("Postcard: bool/option tag".into()),
        }
    }
    fn length(&mut self, cap: u64) -> Result<usize> {
        let len = self.number(64)?;
        if len > cap || len > (self.bytes.len() - self.at) as u64 {
            return Err("Limit: collection/string length".into());
        }
        usize::try_from(len).map_err(|_| "Limit: usize".into())
    }
    fn string(&mut self) -> Result<()> {
        let len = self.length(self.limits.string_max_bytes)?;
        self.charge(
            (len as u64)
                .checked_mul(2)
                .ok_or("Limit: string allocation")?,
        )?;
        let end = self.at.checked_add(len).ok_or("Limit: string offset")?;
        std::str::from_utf8(&self.bytes[self.at..end]).map_err(|_| "Postcard: UTF-8")?;
        self.at = end;
        Ok(())
    }
    fn date(&mut self) -> Result<()> {
        self.number(32)?;
        self.byte()?;
        self.byte()?;
        Ok(())
    }
    fn optional_number(&mut self, bits: u32) -> Result<()> {
        if self.boolean()? {
            self.number(bits)?;
        }
        Ok(())
    }
    fn map(&mut self) -> Result<()> {
        for _ in 0..self.entries(self.limits.map_entries_max)? {
            self.string()?;
            self.number(64)?;
        }
        Ok(())
    }
    fn end(self) -> Result<()> {
        if self.at != self.bytes.len() {
            Err("Trailing: postcard".into())
        } else {
            Ok(())
        }
    }
}
pub fn header(bytes: &[u8], limits: &Limits) -> Result<()> {
    if bytes.len() as u64 > limits.header_max_bytes {
        return Err("Limit: header".into());
    }
    let mut r = Reader {
        bytes,
        at: 0,
        limits,
        charged: 0,
    };
    r.string()?;
    r.number(16)?;
    r.string()?;
    for _ in 0..r.entries(limits.packs_max)? {
        r.string()?;
        r.string()?;
        r.number(64)?;
    }
    r.date()?;
    for _ in 0..3 {
        r.number(64)?;
    }
    for _ in 0..r.entries(65536)? {
        r.number(16)?;
    }
    r.number(64)?;
    r.optional_number(64)?;
    r.number(64)?;
    r.end()
}
pub fn body(bytes: &[u8], limits: &Limits) -> Result<()> {
    body_version(bytes, limits, 1)
}
pub fn body_version(bytes: &[u8], limits: &Limits, version: u16) -> Result<()> {
    if bytes.len() as u64 > limits.body_max_bytes {
        return Err("Limit: body".into());
    }
    let mut r = Reader {
        bytes,
        at: 0,
        limits,
        charged: 0,
    };
    r.string()?;
    r.number(64)?;
    r.number(64)?;
    r.date()?;
    r.byte()?;
    r.boolean()?;
    r.byte()?;
    for _ in 0..5 {
        r.number(64)?;
    }
    r.byte()?;
    for _ in 0..r.entries(limits.queue_max_entries)? {
        r.number(64)?;
        r.number(16)?;
        r.number(64)?;
        match r.tag(1)? {
            0 => {
                r.boolean()?;
            }
            1 => {
                r.byte()?;
            }
            _ => unreachable!(),
        }
    }
    if r.boolean()? {
        r.number(64)?;
        for _ in 0..r.entries(65536)? {
            r.number(16)?;
            r.string()?;
            r.string()?;
            r.map()?;
        }
        for _ in 0..r.entries(65536)? {
            r.number(16)?;
            r.number(16)?;
            r.number(64)?;
            r.map()?;
            r.map()?;
            r.number(64)?;
            for _ in 0..r.entries(limits.modifiers_per_state)? {
                r.string()?;
                r.string()?;
                r.tag(1)?;
                r.number(64)?;
                r.optional_number(64)?;
            }
            r.number(64)?;
            r.string()?;
            r.number(64)?;
            r.number(64)?;
            for _ in 0..r.entries(
                limits
                    .modifiers_per_state
                    .checked_add(1)
                    .ok_or("Limit: ledger cap")?,
            )? {
                if r.boolean()? {
                    r.string()?;
                }
                r.tag(2)?;
                r.number(64)?;
                r.number(64)?;
            }
        }
        for _ in 0..r.entries(65536)? {
            r.number(16)?;
            r.optional_number(16)?;
            r.optional_number(16)?;
            r.optional_number(16)?;
        }
    }
    if version >= 2 {
        for _ in 0..r.entries(limits.queue_max_entries)? {
            r.number(64)?;
            r.number(16)?;
            r.number(64)?;
            match r.tag(3)? {
                0 => {
                    r.boolean()?;
                }
                1 => {
                    r.byte()?;
                }
                2 => {
                    r.number(32)?;
                    r.number(16)?;
                }
                3 => {
                    r.number(32)?;
                }
                _ => unreachable!(),
            }
        }
        for _ in 0..r.entries(limits.map_entries_max)? {
            r.number(32)?;
            r.number(16)?;
            r.number(16)?;
            r.number(64)?;
            for _ in 0..r.entries(65536)? {
                r.number(16)?;
            }
            for _ in 0..r.entries(limits.map_entries_max)? {
                r.number(16)?;
                r.number(16)?;
                for _ in 0..4 {
                    r.number(64)?;
                }
            }
            for _ in 0..r.entries(limits.map_entries_max)? {
                r.number(16)?;
                r.number(16)?;
                r.number(64)?;
            }
            r.number(64)?;
        }
    }
    if version == 3 || version == 4 {
        for _ in 0..r.entries(limits.map_entries_max)? {
            r.number(32)?;
            for _ in 0..r.entries(limits.map_entries_max)? {
                r.number(16)?;
                r.number(16)?;
                if r.byte()? != 3 {
                    return Err("Postcard: strait crossing kind".into());
                }
                for _ in 0..4 {
                    r.number(64)?;
                }
            }
        }
    }
    if version == 4 {
        r.boolean()?; // movement presence
        r.boolean()?; // explicit strait presence
        for _ in 0..r.entries(limits.queue_max_entries)? {
            r.number(64)?;
            r.number(16)?;
            r.number(64)?;
            match r.tag(4)? {
                0 => {
                    r.boolean()?;
                }
                1 => {
                    r.byte()?;
                }
                2 => {
                    r.number(32)?;
                    r.number(16)?;
                }
                3 => {
                    r.number(32)?;
                }
                4 => {
                    r.string()?;
                }
                _ => unreachable!(),
            }
        }
        r.number(64)?; // trigger definitions identity
        for _ in 0..r.entries(65536)? {
            r.number(16)?;
            for _ in 0..r.entries(limits.map_entries_max)? {
                r.string()?;
            }
        }
        if r.boolean()? {
            r.number(64)?;
            r.date()?;
            r.byte()?;
            for _ in 0..r.entries(limits.map_entries_max)? {
                if r.tag(2)? == 2 {
                    r.string()?;
                }
            }
        }
    }
    r.end()
}
