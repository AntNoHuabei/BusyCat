"""Transparent phone reaction overlays; original code-drawn vector artwork."""
from pathlib import Path
import io
import pymupdf as fitz
from PIL import Image, ImageDraw
ROOT=Path(__file__).resolve().parents[1]
OUT=ROOT/'assets'/'cat'

def face(kind):
 s=['<svg xmlns="http://www.w3.org/2000/svg" width="512" height="512">']
 def ellipse(x,y,rx,ry,c):s.append(f'<ellipse cx="{x}" cy="{y}" rx="{rx}" ry="{ry}" fill="{c}"/>')
 def path(d,c='#a9a7a5',w=2):s.append(f'<path d="{d}" fill="none" stroke="{c}" stroke-width="{w}" stroke-linecap="round"/>')
 for x in [211,300]:
  ellipse(x,211,34,36,'#252e3a')
  if kind=='happy' or (kind=='blush' and x==211):
   path(f'M {x-22} 217 Q {x} 194 {x+22} 217','#c7e782',4)
  else:
   h=31 if kind in ['surprise','blush'] else 14 if kind=='unimpressed' else 9
   ellipse(x,215,27,h,'#c7e782')
   ellipse(x+3,219,7 if kind=='surprise' else 9,h*.74,'#15202b')
   if kind=='surprise':ellipse(x+6,207,3,4,'#f4ffdf')
   if kind=='unimpressed':path(f'M {x-26} 200 L {x+25} 203','#53616d',3)
 if kind in ['happy','blush']:
  for x in [184,327]:ellipse(x,243,12,5,'#ae777b')
 ellipse(256,252,19,10,'#252e3a')
 if kind=='surprise':ellipse(256,252,5,7,'#121923')
 elif kind=='sleepy':ellipse(256,253,7,8,'#121923')
 elif kind=='unimpressed':path('M 247 254 Q 256 250 265 254')
 else:path('M 244 248 Q 256 264 268 248','#c7a4a0',2.5)
 if kind=='blush':
  path('M 337 230 l 3 -4 m 1 8 l 3 -4','#dda4a5',2)
 s.append('</svg>');return ''.join(s)

def raster(svg):
 doc=fitz.open(stream=svg.encode(),filetype='svg');pdf=fitz.open('pdf',doc.convert_to_pdf())
 pix=pdf[0].get_pixmap(matrix=fitz.Matrix(2,2),alpha=True)
 return Image.open(io.BytesIO(pix.tobytes('png'))).convert('RGBA').resize((512,512),Image.Resampling.LANCZOS)
if __name__=='__main__':
 sheet=Image.new('RGB',(1280,300),'#e9e5dc')
 base=Image.open(OUT/'phone_hold.png').convert('RGBA')
 for i,k in enumerate(['happy','surprise','unimpressed','sleepy','blush']):
  svg=face(k);(OUT/f'phone_{k}_face.svg').write_text(svg,encoding='utf-8')
  img=raster(svg);img.save(OUT/f'phone_{k}_face.png')
  sample=Image.alpha_composite(base,img).resize((256,256));sheet.paste(sample,(i*256,0),sample)
  ImageDraw.Draw(sheet).text((i*256+90,270),k,fill='#34424d')
 sheet.save(ROOT/'preview'/'phone-expressions.png')
