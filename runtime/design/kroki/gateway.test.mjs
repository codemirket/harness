import test from 'node:test';
import assert from 'node:assert/strict';
import {permitted} from './gateway.mjs';
test('service boundary rejects unused engines, URL encodings and administrative routes',()=>{
 for(const type of ['plantuml','c4plantuml','vegalite','excalidraw','d2','graphviz']) assert.equal(permitted('POST',`/${type}/svg`),true);
 for(const url of ['/tikz/svg','/mermaid/svg','/services','/plantuml/svg/extra','/plantuml/svg?url=http://host','/%70lantuml/svg','/plantuml/pdf','/files/private']) assert.equal(permitted('POST',url),false,url);
 assert.equal(permitted('GET','/plantuml/svg'),false);assert.equal(permitted('GET','/health'),true);
});
