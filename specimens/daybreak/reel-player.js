"use strict";
const video=document.getElementById('reel');
const status=document.getElementById('play-status');
document.getElementById('play').addEventListener('click',async()=>{
 video.currentTime=0;
 try{await video.play();status.textContent='Playing the silent film.'}catch{status.textContent='Playback could not start. Use the video controls or download link.'}
});
for(const button of document.querySelectorAll('[data-time]'))button.addEventListener('click',()=>{
 video.pause();video.currentTime=Number(button.dataset.time);status.textContent=button.textContent+' — paused for inspection.';
});
video.addEventListener('ended',()=>{status.textContent='Film finished.'});
