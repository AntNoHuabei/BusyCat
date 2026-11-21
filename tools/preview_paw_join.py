"""Render the same two-pass limb geometry as the native app for wrist visual checks."""
from pathlib import Path
import io,json
from PIL import Image,ImageDraw
import pymupdf
ROOT=Path(__file__).resolve().parents[1]
base=Image.open(ROOT/'assets/cat/atlas.png').convert('RGBA').crop((0,0,512,512))
mouse=Image.open(ROOT/'assets/cat/mouse.png').convert('RGBA')
keys=json.loads((ROOT/'assets/cat/keyboard-layout.json').read_text())
def limb(curve,center):
 x,y=center;s=['<svg xmlns="http://www.w3.org/2000/svg" width="512" height="512">']
 for border,c in [(True,'#141d28'),(False,'#252e3a')]:
  w=34 if border else 29
  d=f'M {curve[0][0]} {curve[0][1]} C '+ ' '.join(f'{a} {b}' for a,b in curve[1:])
  s.append(f'<path d="{d}" fill="none" stroke="{c}" stroke-width="{w}" stroke-linecap="butt"/>')
  a,b=curve[-1];s.append(f'<circle cx="{a}" cy="{b}" r="{w/2}" fill="{c}"/>')
  s.append(f'<ellipse cx="{x}" cy="{y-1}" rx="{18 if border else 15.5}" ry="{13 if border else 10.5}" fill="{c}"/>')
  for dx in [-10.5,-3.5,3.5,10.5]:s.append(f'<circle cx="{x+dx}" cy="{y+6-(1 if abs(dx)>10 else 0)}" r="{5 if border else 3.5}" fill="{c}"/>')
 for dx in [-7,0,7]:s.append(f'<path d="M {x+dx} {y+6} v1.8" stroke="#384451" stroke-width=".8"/>')
 s.append('</svg>');doc=pymupdf.open(stream=''.join(s).encode(),filetype='svg');pdf=pymupdf.open('pdf',doc.convert_to_pdf());pix=pdf[0].get_pixmap(matrix=pymupdf.Matrix(2,2),alpha=True)
 return Image.open(io.BytesIO(pix.tobytes('png'))).convert('RGBA').resize((512,512),Image.Resampling.LANCZOS)
sheet=Image.new('RGB',(1536,560),'#eee9df')
for j,vk in enumerate([None,65,76]):
 im=Image.new('RGBA',(512,512));im.alpha_composite(mouse);im.alpha_composite(base)
 for side in [True,False]:
  shoulder=195 if side else 315;x=211 if side else 301;y=354
  if vk and side==(vk==76):
   k=next(k for k in keys if k['vk']==vk);a,b,w,h=k['rect'];x=a+w/2;y=b+h/2-10
  im.alpha_composite(limb([(shoulder,301),(shoulder,340),(x,y-34),(x,y-7)],(x,y)))
 sheet.paste(im,(j*512,0),im)
 ImageDraw.Draw(sheet).text((j*512+230,530),['Rest','A','L'][j],fill='#34424d')
sheet.save(ROOT/'preview/paw-join.png')
