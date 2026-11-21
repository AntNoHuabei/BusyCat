const tailImage=new Image();tailImage.src="../assets/cat/tail.png";
let tailPhase=0,tailAmplitude=12,tailTick=0;
// Interactive local preview. Native Windows global input lives in src/input.rs.
const canvas=document.querySelector('#cat'),ctx=canvas.getContext('2d');
const atlas=new Image(),mouse=new Image();atlas.src='../assets/cat/atlas.png';mouse.src='../assets/cat/mouse.png';
const downAt=[-1e6,-1e6,-1e6],upAt=[-1e6,-1e6,-1e6];
function depth(b,t){const smooth=v=>{const x=Math.max(0,Math.min(1,v));return x*x*(3-2*x);};return smooth((t-downAt[b])/35)*(1-smooth((t-Math.max(upAt[b],downAt[b]+70))/120));}
const keyTimes=new Map();
const held=new Set(),buttons=new Set(),bubbles=[];
let activity=performance.now(),lastKey=-1e6,lastMouse=-1e6,lastClick=-1e6,lastWheel=-1e6,left=-1e6,right=-1e6;
let cursor=[.5,.5],travel=[0,0],demo=null,demoStep=0;
const leftHand=vk=>(vk>=65&&vk<=90)?'QWERTASDFGZXCVB'.includes(String.fromCharCode(vk)):(vk>=49&&vk<=53)||[9,20,27,160,162,164,91].includes(vk);
const keyMap={Escape:27,Backspace:8,Tab:9,Enter:13,CapsLock:20,Space:32,ShiftLeft:160,ShiftRight:161,ControlLeft:162,ControlRight:163,AltLeft:164,AltRight:165,MetaLeft:91,MetaRight:92,Minus:189,Equal:187,BracketLeft:219,BracketRight:221,Backslash:220,Semicolon:186,Quote:222,Comma:188,Period:190,Slash:191};
function vk(e){return e.code.startsWith('Key')?e.code.charCodeAt(3):e.code.startsWith('Digit')?e.code.charCodeAt(5):keyMap[e.code]||0;}
function bubble(text,isMouse=false){bubbles.push({text,mouse:isMouse,t:performance.now(),slot:demoStep++%7});if(bubbles.length>18)bubbles.shift();}
function key(v,down,label){const t=performance.now();activity=t;if(!down){held.delete(v);return;}held.add(v);keyTimes.set(v,t);lastKey=t;if(leftHand(v))right=t;else left=t;bubble(label);}
document.addEventListener('keydown',e=>{if(e.target.tagName==='BUTTON'&&[' ','Enter','Tab'].includes(e.key))return;if(['Tab',' ','ArrowUp','ArrowDown'].includes(e.key))e.preventDefault();const parts=[];if(e.ctrlKey&&!e.key.includes('Control'))parts.push('Ctrl');if(e.altKey&&e.key!=='Alt')parts.push('Alt');if(e.shiftKey&&e.key!=='Shift')parts.push('Shift');parts.push(e.key===' '?'Space':e.key.length===1?e.key.toUpperCase():e.key);key(vk(e),true,parts.join(' + '));});
document.addEventListener('keyup',e=>key(vk(e),false));
document.addEventListener('mousemove',e=>{activity=lastMouse=performance.now();cursor=[e.clientX/innerWidth,e.clientY/innerHeight];});
document.addEventListener('mousedown',e=>{activity=lastMouse=lastClick=performance.now();buttons.add(e.button);downAt[e.button]=performance.now();upAt[e.button]=Infinity;if(e.target===canvas)bubble(['LMB','MMB','RMB'][e.button]||'Mouse',true);});
document.addEventListener('mouseup',e=>{activity=lastMouse=performance.now();buttons.delete(e.button);upAt[e.button]=performance.now();});
canvas.addEventListener('contextmenu',e=>e.preventDefault());
canvas.addEventListener('wheel',e=>{e.preventDefault();activity=lastMouse=lastWheel=performance.now();bubble(e.deltaY<0?'Wheel Up':'Wheel Down',true);},{passive:false});
window.addEventListener('blur',()=>{held.clear();buttons.clear();});
document.querySelectorAll('[data-action]').forEach(b=>b.onclick=()=>{
 const t=performance.now(),a=b.dataset.action;activity=t;
 if(a==='phone'){if(demo){clearInterval(demo);demo=null;document.querySelector('#demo').textContent='连续打字';}held.clear();buttons.clear();activity=t-15000;}
 else if(a==='left'){left=lastKey=t;bubble('J');}else if(a==='right'){right=lastKey=t;bubble('A');}
 else if(a==='both'){left=right=lastKey=t;bubble('A + J');}
 else if(a==='mouse'){lastMouse=lastClick=t;downAt[0]=upAt[0]=t;bubble('LMB',true);}
});
document.querySelector('#demo').onclick=e=>{if(demo){clearInterval(demo);demo=null;e.target.textContent='连续打字';}else{e.target.textContent='停止演示';demo=setInterval(()=>{const v=demoStep%2?65:74;key(v,true,String.fromCharCode(v));key(v,false);},260);}};
function frame(t){
 if(t-activity>=15000&&!held.size&&!buttons.size){const p=(t-activity-15000)%3000;return(p>=700&&p<=1050)||(p>=2200&&p<=2450)?13:p>=1600&&p<=1750?14:12;}
 const lh=[...held].some(v=>!leftHand(v)),rh=[...held].some(leftHand);
 if(t-lastKey>220&&!held.size&&(t-lastMouse<650||buttons.size))return buttons.size||t-lastClick<180||t-lastWheel<170?11:10;
 const l=lh||t-left<180,r=rh||t-right<180;
 if(l||r){const down=lh||rh||Math.min(t-left,t-right)>=30&&Math.min(t-left,t-right)<120;return l&&r?(down?9:8):l?(down?5:4):(down?7:6);}
 const p=t%5400;return p>=4155&&p<4250?3:p>=4100&&p<4305?2:p>=2400&&p<=2850?1:0;
}
const phoneFaces=['happy','surprise','unimpressed','sleepy','blush'].map(k=>{const img=new Image();img.src=`../assets/cat/phone_${k}_face.png`;return img;});
let phoneActive=false,phoneNext=0,phoneStarted=-1e6,phoneIndex=0;
function paintPhoneFace(t,on){
 if(!on){phoneActive=false;return;}
 if(!phoneActive){phoneActive=true;phoneNext=t+900+Math.random()*1300;phoneStarted=-1e6;}
 if(t>=phoneNext){phoneIndex=(phoneIndex+1+Math.floor(Math.random()*4))%5;phoneStarted=t;phoneNext=t+4200+Math.random()*3500;}
 const age=(t-phoneStarted)/1000,alpha=Math.max(0,Math.min(1,age/.18,(2.6-age)/.35)),img=phoneFaces[phoneIndex];
 if(alpha&&img.complete&&img.naturalWidth){ctx.globalAlpha=alpha;ctx.drawImage(img,0,0,512,512);ctx.globalAlpha=1;}
}
function softLimb(curve,[x,y]){
 ctx.lineCap='round';ctx.lineJoin='round';
 for(const[border,c]of[[true,'#141d28'],[false,'#252e3a']]){
  const w=border?34:29;ctx.beginPath();ctx.moveTo(...curve[0]);ctx.bezierCurveTo(...curve[1],...curve[2],...curve[3]);ctx.lineWidth=w;ctx.strokeStyle=c;ctx.stroke();
  ctx.fillStyle=c;ctx.beginPath();ctx.ellipse(x,y-1,border?18:15.5,border?13:10.5,0,0,Math.PI*2);ctx.fill();
  for(const dx of[-10.5,-3.5,3.5,10.5]){ctx.beginPath();ctx.arc(x+dx,y+6-(Math.abs(dx)>10?1:0),border?5:3.5,0,Math.PI*2);ctx.fill();}
 }
 for(const dx of[-7,0,7]){ctx.beginPath();ctx.moveTo(x+dx,y+6);ctx.lineTo(x+dx,y+7.8);ctx.strokeStyle='#384451';ctx.lineWidth=.8;ctx.stroke();}
}
function keyboardPaws(t,i){
 if(i>=12)return;
 for(const side of[true,false]){
  if(side&&(i===10||i===11))continue;
  const candidates=[...keyTimes].filter(([v,at])=>!leftHand(v)===side&&v!==119&&(held.has(v)||t-at<210)&&window.busycatKeys.some(k=>k.vk===v));
  candidates.sort((a,b)=>(held.has(b[0])-held.has(a[0]))||b[1]-a[1]||b[0]-a[0]);
  const picked=candidates[0],shoulder=side?204:308;let x=side?211:301,y=354,d=0;
  if(picked){const [v,at]=picked,[kx,ky,w,h]=window.busycatKeys.find(k=>k.vk===v).rect,age=(t-at)/1000;d=Math.max(0,Math.min(1,age/.03,held.has(v)?1:(.21-age)/.07));x=kx+w/2;y=ky+h/2-10-(1-d)*18;}
  softLimb([[shoulder,288],[shoulder,340],[x,y-34],[x,y-7]],[x,y]);
 }
}
function paint(t){
 const i=frame(t);travel[0]+=((.5-cursor[0])*36-travel[0])*.16;travel[1]+=((.5-cursor[1])*24-travel[1])*.16;
 ctx.clearRect(0,0,512,512);
 const dt=Math.min(.1,tailTick?(t-tailTick)/1000:0);tailTick=t;
 const amp=i>=12?9:i>=4&&i<=9?19:14,speed=i>=12?1.25:i>=4&&i<=9?2.5:1.75;
 tailAmplitude+=(amp-tailAmplitude)*(1-Math.exp(-dt*3));tailPhase=(tailPhase+dt*speed)%(Math.PI*2);
 if(tailImage.complete&&tailImage.naturalWidth)for(let x=320;x<480;x+=2){const w=Math.max(0,Math.min(1,(x-344)/112))**2,dy=(Math.sin(tailPhase)+.18*Math.sin(tailPhase*2-.8))*tailAmplitude*w,dx=Math.cos(tailPhase)*3*w;ctx.drawImage(tailImage,x,256,2,154,x+dx,256+dy,2.5,154);}

 if(mouse.complete&&mouse.naturalWidth)ctx.drawImage(mouse,travel[0],travel[1],512,512);
 for(const b of[0,2]){const d=depth(b,t),outline=[[49.5,411.5],[68,411.5],[69,422],[68,430],[64,436],[57,439.5],[49.5,440.5]];for(const [dy,color]of[[1,'#6e7774'],[-1.2+d*2.4,`rgb(${239-d*17},${234-d*17},${221-d*17})`]]){ctx.beginPath();outline.forEach(([x,y],j)=>ctx[j?'lineTo':'moveTo']((b===0?x:96-x)+travel[0],y+dy+travel[1]));ctx.closePath();ctx.fillStyle=color;ctx.fill();ctx.strokeStyle='#a1aaa3';ctx.lineWidth=.8;ctx.stroke();}}
 if(atlas.complete&&atlas.naturalWidth)ctx.drawImage(atlas,(i%6)*512,Math.floor(i/6)*512,512,512,0,0,512,512);
 if(i===10||i===11){const l=depth(0,t),r=depth(2,t),x=48+travel[0]+(l-r)*1.3,y=423+travel[1]+Math.max(l,r)*3;softLimb([[204,288],[172,345],[x+46,y-26],[x,y-7]],[x,y]);}

 // Look at the phone while idle, otherwise follow the cursor.
 const openness=i===3?0:i===2?.45:i===14?.3:[5,7,9,13].includes(i)?.85:1;
 if(openness){for(const x of[211,300]){ctx.beginPath();ctx.ellipse(x,211,29,31*openness,0,0,Math.PI*2);ctx.fillStyle='#c7e782';ctx.fill();const px=x+4+(i>=12?0:(cursor[0]-.5)*18),py=214+(i>=12?6:(cursor[1]-.5)*9)*openness;ctx.beginPath();ctx.ellipse(px,py,10,22*openness,0,0,Math.PI*2);ctx.fillStyle='#15202b';ctx.fill();ctx.beginPath();ctx.ellipse(px+4,py-11*openness,4,5*openness,0,0,Math.PI*2);ctx.fillStyle='#f4ffdf';ctx.fill();}}
 paintPhoneFace(t,i>=12);
 for(const k of window.busycatKeys||[]){if(held.has(k.vk)){ctx.fillStyle='#f1b03a';ctx.fillRect(...k.rect);}}
 keyboardPaws(t,i);
 if(i<12&&atlas.complete&&atlas.naturalWidth){
  for(let y=268;y<327;y++){
   const alpha=y<=302?1:y<=313?1-(y-302)/11*.35:.65*(327-y)/14;
   ctx.globalAlpha=alpha;ctx.drawImage(atlas,(i%6)*512+164,Math.floor(i/6)*512+y,184,1,164,y,184,1);
  }ctx.globalAlpha=1;
 }
 for(let j=bubbles.length-1;j>=0;j--){const b=bubbles[j],age=(t-b.t)/1250;if(age>=1){bubbles.splice(j,1);continue;}ctx.globalAlpha=Math.sqrt(1-age);ctx.font='17px system-ui';const w=ctx.measureText(b.text).width+16;let x=(b.mouse?.1+b.slot%3*.055:.24+b.slot*.075)*512-w/2;x=Math.max(0,Math.min(512-w,x));const y=338-age*159;ctx.fillStyle='#f5f0df';ctx.beginPath();ctx.roundRect(x,y,w,29,5);ctx.fill();ctx.fillStyle='#202733';ctx.fillText(b.text,x+8,y+20);ctx.globalAlpha=1;}
 requestAnimationFrame(paint);
}
requestAnimationFrame(paint);

