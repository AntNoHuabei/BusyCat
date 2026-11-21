from pathlib import Path
import math, io, json
from PIL import Image, ImageDraw, ImageChops
import pymupdf
ROOT=Path(__file__).resolve().parents[1]
ns={'__file__':str(ROOT/'tools/preview_paw_join.py')}
exec((ROOT/'tools/preview_paw_join.py').read_text().split('sheet=Image.new')[0],ns)
limb=ns['limb']; atlas=Image.open(ROOT/'assets/cat/atlas.png').convert('RGBA'); mouse=ns['mouse']; keys=ns['keys']
tail=Image.open(ROOT/'assets/cat/tail.png').convert('RGBA')
frames=[]
for n in range(132):
 t=n/12; phone=t>=7; moving=4<=t<7
 idx=(13 if n%24<5 else 12) if phone else 0
 base=atlas.crop(((idx%6)*512,(idx//6)*512,(idx%6+1)*512,(idx//6+1)*512))
 im=Image.new('RGBA',(512,512),'#eee9df')
 for x in range(320,480,2):
  w=max(0,min(1,(x-344)/112))**2
  dy=round((math.sin(t*2)+.18*math.sin(t*4-.8))*(9 if phone else 16)*w)
  im.alpha_composite(tail.crop((x,256,x+2,410)),(x,256+dy))
 dx=round(math.sin(t*2)*17) if moving else 0;dy=round(math.cos(t*2)*10) if moving else 0
 im.alpha_composite(mouse,(dx,dy)); im.alpha_composite(base)
 if not phone:
  active=next(k for k in keys if k['vk']==[65,74,83,75,68,76,32,13][int(t*4)%8]) if t<4 else None
  for side in [True,False]:
   shoulder=204 if side else 308;x=211 if side else 301;y=354
   if moving and side:
    x,y=48+dx,423+dy
    curve=[(204,288),(172,345),(x+46,y-26),(x,y-7)]
   else:
    if active and side==(active['vk'] in [74,75,76,13]):
     a,b,w,h=active['rect'];x=a+w/2;y=b+h/2-10-9*(1-math.sin((t*4%1)*math.pi))
    curve=[(shoulder,288),(shoulder,340),(x,y-34),(x,y-7)]
   im.alpha_composite(limb(curve,(x,y)))
  patch=base.crop((164,268,348,327));mask=Image.new('L',patch.size);d=ImageDraw.Draw(mask)
  for yy in range(59):
   y=yy+268;a=1 if y<=302 else 1-(y-302)/11*.35 if y<=313 else .65*(327-y)/14
   d.line((0,yy,184,yy),fill=round(a*255))
  patch.putalpha(ImageChops.multiply(patch.getchannel('A'),mask));im.alpha_composite(patch,(164,268))
  if active:
   label={32:'SPACE',13:'ENTER'}.get(active['vk'],chr(active['vk']));d=ImageDraw.Draw(im);d.rounded_rectangle((235,45,295,75),6,fill='#f5f0df');d.text((246,53),label,fill='#202733')
 if phone and 8<t<10.8:
  face=Image.open(ROOT/'assets/cat/phone_happy_face.png').convert('RGBA');im.alpha_composite(face)
 d=ImageDraw.Draw(im);label='TYPING' if t<4 else 'MOUSE' if moving else 'IDLE / PHONE';d.text((20,26),label,fill='#455260')
 frames.append(im.convert('RGB').resize((448,448),Image.Resampling.LANCZOS).quantize(colors=128))
frames[0].save(ROOT/'preview/demo.gif',save_all=True,append_images=frames[1:],duration=80,loop=0,optimize=True)

print((ROOT/'preview/demo.gif').stat().st_size)

