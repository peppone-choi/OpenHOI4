from fontTools.ttLib import TTFont
from pathlib import Path
import json,re
r=Path('E:/openhoi/.orchestrator/wt/WP-12-verify2');f=TTFont(r/'client/public/fonts/NotoSansKR.ttf');c=f.getBestCmap()
s=(r/'client/public/locales/ko.ftl').read_text(encoding='utf-8');chars={ch for line in s.splitlines() if ' = ' in line for ch in line.split(' = ',1)[1] if ord(ch)>127 and not ch.isspace()}
missing=[ch for ch in chars if ord(ch) not in c];hangul=[n for n in range(0xAC00,0xD7A4) if n not in c]
keys={lang:set(re.findall(r'^([a-z][a-z0-9-]+) =', (r/f'client/public/locales/{lang}.ftl').read_text(encoding='utf-8'),re.M)) for lang in ['ko','en']}
names={n.nameID:n.toUnicode() for n in f['name'].names if n.nameID in [1,2,5,6]}
result={'catalog_characters':len(chars),'missing':missing,'Hangul_missing':hangul,'CJK_sample':{ch:ord(ch) in c for ch in '漢字'},'names':names,'axes':[{ 'tag':a.axisTag,'min':a.minValue,'default':a.defaultValue,'max':a.maxValue} for a in f['fvar'].axes], 'ko_only_keys':list(keys['ko']-keys['en']),'en_only_keys':list(keys['en']-keys['ko']), 'key_count':len(keys['ko'])}
(r/'target/wp12-verify2/font-glyphs.json').write_text(json.dumps(result,ensure_ascii=False,indent=2),encoding='utf-8');print(json.dumps(result,ensure_ascii=True));assert not missing and not hangul and keys['ko']==keys['en']

