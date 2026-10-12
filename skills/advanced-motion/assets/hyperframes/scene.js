// Finite GSAP score owned by HyperFrames during capture.
// The composition remains ordinary editable DOM and inline SVG.
const score = gsap.timeline({paused:true});
score.fromTo('.wash',{opacity:.25},{opacity:1,duration:6,ease:'none'},0);
const reveal = (selector, at, duration, distance) => score.fromTo(selector,
  {opacity:0,y:distance}, {opacity:1,y:0,duration,ease:'power3.out'}, at);
score.fromTo('.rule',{scaleX:0},{scaleX:1,duration:.9,ease:'power3.out'},0);
reveal('.eyebrow',.15,.8,8);
reveal('h1',.3,1.3,24);
reveal('.description',1.1,.9,12);
reveal('.instrument',0,1.1,18);
reveal('footer',2.1,1,6);
score.fromTo('#sun',{attr:{transform:'translate(0 72)'}},
  {attr:{transform:'translate(0 -60)'},duration:3.4,ease:'power3.out'},.45);
score.fromTo('#pointer',{attr:{transform:'rotate(-48 230 230)'}},
  {attr:{transform:'rotate(46 230 230)'},duration:4.2,ease:'power3.out'},0);
score.to({}, {duration:1.8},4.2);
window.__timelines = {main:score};
// The preview query is an external viewing mode, with no diagnostic UI in the film.
if (new URLSearchParams(location.search).has('preview')) {
  if (matchMedia('(prefers-reduced-motion: reduce)').matches) score.progress(1);
  else score.play(0);
}
