import pathlib,json
p=pathlib.Path('target/wp11-verify2/reference.py');s=p.read_text().replace("a['points'][2]['dto']['state']['speed']==4","a['points'][2]['dto']['state']['speed']==5 and a['points'][2]['dto']['queue'][0]['tick']==25 and end['dto']['state']['speed']==4");p.write_text(s)
pathlib.Path('target/wp11-verify2/reference-rerun.json').write_text(json.dumps([['independent-reference-corrected',['python',str(p)]]]))
