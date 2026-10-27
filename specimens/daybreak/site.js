'use strict';
const filters = [...document.querySelectorAll('[data-filter]')];
const runs = [...document.querySelectorAll('.run-row')];
const filterStatus = document.getElementById('filter-status');
const reduced = matchMedia('(prefers-reduced-motion: reduce)');
const motionHandles = new Map();
function entrance(node) {
  motionHandles.get(node)?.cancel();
  if (reduced.matches) return;
  const animation = node.animate([{opacity:0,transform:'translateY(7px)'},{opacity:1,transform:'translateY(0px)'}],{duration:180,easing:'cubic-bezier(.2,.7,.2,1)'});
  motionHandles.set(node,animation);
  animation.onfinish=()=>motionHandles.delete(node);
}
for(const button of filters)button.addEventListener('click',()=>{
  for(const b of filters)b.setAttribute('aria-pressed',String(b===button));
  let count=0;
  for(const run of runs){motionHandles.get(run)?.cancel();motionHandles.delete(run);run.hidden=button.dataset.filter!=='all'&&run.dataset.pace!==button.dataset.filter;if(!run.hidden){count++;entrance(run)}}
  filterStatus.textContent=`${count} ${count===1?'run':'runs'} available`;
});
reduced.addEventListener('change',()=>{for(const a of motionHandles.values())a.cancel();motionHandles.clear()});
const dialog=document.getElementById('booking'),form=document.getElementById('booking-form');
let lastTrigger,selectedRun='';
function closeBooking(){dialog.close();lastTrigger?.focus()}
for(const b of document.querySelectorAll('[data-run]'))b.addEventListener('click',()=>{
 lastTrigger=b;selectedRun=b.dataset.run;document.getElementById('selected-run').textContent=selectedRun;document.getElementById('selected-time').textContent=b.dataset.time;
 document.getElementById('booking-form-view').hidden=false;document.getElementById('booking-success').hidden=true;dialog.setAttribute('aria-labelledby','booking-title');form.reset();dialog.showModal();document.getElementById('first-name').focus();
});
for(const b of document.querySelectorAll('.close-dialog,.close-success'))b.addEventListener('click',closeBooking);
dialog.addEventListener('cancel',event=>{event.preventDefault();closeBooking()});
dialog.addEventListener('click',event=>{if(event.target===dialog){const r=dialog.getBoundingClientRect();if(event.clientX<r.left||event.clientX>r.right||event.clientY<r.top||event.clientY>r.bottom)closeBooking()}});
form.addEventListener('submit',event=>{
 event.preventDefault();const name=document.getElementById('first-name').value.trim();if(!name){document.getElementById('first-name').setCustomValidity('Please enter your first name.');document.getElementById('first-name').reportValidity();return}
 document.getElementById('confirmation').textContent=`${name}, your demo spot for ${selectedRun} is saved for this visit.`;document.getElementById('booking-form-view').hidden=true;document.getElementById('booking-success').hidden=false;dialog.setAttribute('aria-labelledby','success-title');document.querySelector('.close-success').focus();
});
document.getElementById('first-name').addEventListener('input',event=>event.target.setCustomValidity(''));
window.addEventListener('pagehide',()=>{for(const a of motionHandles.values())a.cancel();motionHandles.clear()});
