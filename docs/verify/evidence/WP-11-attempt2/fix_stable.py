import pathlib
p=pathlib.Path('target/wp11-verify2/stable_prepare.py');s=p.read_text().replace("str(o/'public-file')]]))","str(o/'public-file')]]]))");p.write_text(s)
