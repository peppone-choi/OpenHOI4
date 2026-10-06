from run import *
import shutil
p=OUT/'own/pack';shutil.copytree(ROOT/'data/packs/testland',p)
def change(file,pairs):
 f=p/file;s=f.read_text(encoding='utf-8')
 for a,b in pairs:assert a in s,(file,a);s=s.replace(a,b)
 f.write_text(s,encoding='utf-8',newline='\n')
change('scenarios/m1/nations/NTH.toml',[('id = 1','id = 0'),('capital = 10','capital = 0')])
change('scenarios/m1/nations/STH.toml',[('id = 2','id = 65535'),('capital = 30','capital = 65535')])
change('maps/testland/provinces.csv',[('10,200','0,200'),('20,40,200','13,40,200'),('30,40,40','65535,40,40'),('40,200,200','91,200,200')])
change('maps/testland/states.toml',[('id = 1','id = 0'),('id = 2','id = 901'),('[10, 20]','[0, 13]'),('[30, 40]','[65535, 91]'),('province = 10','province = 0'),('province = 30','province = 65535')])
change('scenarios/m1/scenario.toml',[('1 = "NTH"','0 = "NTH"'),('2 = "STH"','901 = "STH"'),('20 = "STH"','13 = "STH"')])
change('maps/testland/visuals.toml',[('1 = [140','0 = [140'),('2 = [185','901 = [185')])
(p/'maps/testland/adjacency_overrides.csv').write_text('a,b,kind\n0,13,river_small\n91,65535,river_large\n0,91,strait\n50,91,impassable\n',encoding='utf-8')
(OUT/'generated').mkdir(exist_ok=True)
print('fresh independent pack prepared; native validated loader will check it')
