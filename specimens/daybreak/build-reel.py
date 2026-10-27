"""Build the film's three shots from the actual page sections, without a runtime dependency."""
from pathlib import Path
import re
root=Path(__file__).resolve().parent
page=(root/'index.html').read_text()
def section(ident):
    result=re.search(r'<section\b[^>]*\bid="'+ident+r'"[^>]*>[\s\S]*?</section>',page)
    if not result:raise ValueError(ident)
    return result.group()
hero,club,sessions=[section(i) for i in ['home','club','sessions']]
# IDs remain unique because each source section appears once, including on return.
film='''<!doctype html><html lang="en"><head><meta charset="utf-8"><title>Daybreak — design film</title><link rel="stylesheet" href="./site.css"><link rel="stylesheet" href="./film.css"></head><body class="film"><main data-composition-id="daybreak" data-no-timeline data-width="1280" data-height="720" data-duration="12"><div id="intro"><span id="intro-label">GOOD MORNINGS START TOGETHER</span><div id="intro-word">daybreak</div><span id="intro-rule"></span></div><div id="shot-hero" class="shot">'''+hero+'''</div><div id="shot-club" class="shot">'''+club+'''</div><div id="shot-sessions" class="shot">'''+sessions+'''</div><div id="film-label">DAYBREAK / A CITY RUNNING CLUB</div></main><script src="./film.js"></script></body></html>'''
(root/'film.html').write_text(film)
print('Built film.html from the three current page sections')
