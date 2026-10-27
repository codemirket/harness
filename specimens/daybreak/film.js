'use strict';
// All times are absolute milliseconds. HyperFrames owns export seeking.
// Named native WAAPI tracks, one property owner per target. No timers/media clocks.
const DURATION=12000;
const OUT='cubic-bezier(.22,.78,.18,1)', INOUT='cubic-bezier(.65,0,.25,1)';
function track(selector,keys){
 const node=document.querySelector(selector);if(!node)throw Error('Missing film target: '+selector);
 if(!node.id)node.id='film-'+selector.replace(/[^a-zA-Z0-9]/g,'-');
 for(const[,values]of keys)if(values.transform&&!CSS.supports('transform',values.transform))throw Error('Invalid transform: '+values.transform);
 const animation=node.animate(keys.map(([time,values,easing])=>({offset:time/DURATION,...values,easing:easing||OUT})),{duration:DURATION,fill:'both',iterations:1,easing:'linear'});animation.id=node.id;return animation;
}
track('#intro',[[0,{opacity:1}],[750,{opacity:1}],[1450,{opacity:0}],[12000,{opacity:0}]]);
track('#intro-word',[[0,{clipPath:'inset(0 100% 0 0)',transform:'translateY(18px)'}],[650,{clipPath:'inset(0 0% 0 0)',transform:'translateY(0px)'}],[1200,{clipPath:'inset(0 0% 0 0)',transform:'translateY(-10px)'}],[12000,{clipPath:'inset(0 0% 0 0)',transform:'translateY(-10px)'}]]);
track('#intro-label',[[0,{opacity:0}],[250,{opacity:0}],[600,{opacity:1}],[12000,{opacity:1}]]);
track('#intro-rule',[[0,{transform:'scaleX(0)'}],[600,{transform:'scaleX(1)'}],[12000,{transform:'scaleX(1)'}]]);
track('#shot-hero',[[0,{opacity:0,transform:'translate(0px,48px) scale(.86)',clipPath:'inset(12% 18% 12% 18% round 24px)'}],[850,{opacity:0,transform:'translate(0px,48px) scale(.86)',clipPath:'inset(12% 18% 12% 18% round 24px)'}],[1750,{opacity:1,transform:'translate(0px,0px) scale(1)',clipPath:'inset(0% 0% 0% 0% round 0px)'}],[12000,{opacity:1,transform:'translate(0px,0px) scale(1)',clipPath:'inset(0% 0% 0% 0% round 0px)'}]]);
track('#shot-hero .hero-photo',[[0,{transform:'scale(1.07)'}],[1750,{transform:'scale(1.03)'}],[3450,{transform:'scale(1)'}],[12000,{transform:'scale(1)'}]]);
track('#shot-hero .hero-copy',[[0,{opacity:0,transform:'translateY(20px)'}],[1250,{opacity:0,transform:'translateY(20px)'}],[2050,{opacity:1,transform:'translateY(0px)'}],[12000,{opacity:1,transform:'translateY(0px)'}]]);
track('#shot-hero .wordmark',[[0,{clipPath:'inset(0 100% -40% 0)'}],[1400,{clipPath:'inset(0 100% -40% 0)'}],[2200,{clipPath:'inset(0 0% -40% 0)'}],[12000,{clipPath:'inset(0 0% -40% 0)'}]]);
// Opaque panels cover the previous shot; no translucent text or exposed stage.
track('#shot-club',[[0,{transform:'translateX(1280px)'}],[3450,{transform:'translateX(1280px)'},INOUT],[4500,{transform:'translateX(0px)'}],[9750,{transform:'translateX(0px)'},INOUT],[10700,{transform:'translateY(-720px)'}],[12000,{transform:'translateY(-720px)'}]]);
track('#shot-club .club-photo-wrap',[[0,{clipPath:'inset(0 0 100% 0 round 12px)'}],[3250,{clipPath:'inset(0 0 100% 0 round 12px)'}],[4000,{clipPath:'inset(0 0 0% 0 round 12px)'}],[12000,{clipPath:'inset(0 0 0% 0 round 12px)'}]]);
track('#shot-club .pace-card',[[0,{opacity:0,transform:'translateY(20px)'}],[3550,{opacity:0,transform:'translateY(20px)'}],[4200,{opacity:1,transform:'translateY(0px)'}],[12000,{opacity:1,transform:'translateY(0px)'}]]);
const route=document.querySelector('#shot-club .route-line path');route.id='film-route';const length=route.getTotalLength();route.style.strokeDasharray=String(length);
track('#film-route',[[0,{strokeDashoffset:String(length)}],[4800,{strokeDashoffset:String(length)}],[5900,{strokeDashoffset:'0'}],[12000,{strokeDashoffset:'0'}]]);
track('#shot-sessions',[[0,{transform:'translateY(720px)'}],[6650,{transform:'translateY(720px)'},INOUT],[7700,{transform:'translateY(0px)'}],[9750,{transform:'translateY(0px)'},INOUT],[10700,{transform:'translateY(-720px)'}],[12000,{transform:'translateY(-720px)'}]]);
for(const[i,row]of [...document.querySelectorAll('#shot-sessions .run-row')].entries()){
 row.id='film-run-'+i;const t=7450+i*130;
 track('#'+row.id,[[0,{opacity:0,transform:'translateY(16px)'}],[t,{opacity:0,transform:'translateY(16px)'}],[t+450,{opacity:1,transform:'translateY(0px)'}],[12000,{opacity:1,transform:'translateY(0px)'}]]);
}
