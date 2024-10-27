function g(e,t,o){let r=V(e).replace(`#icon-${t}`,`#icon-${o}`);Y(e,r)}function V(e){let t=e.getAttributeNS(_,"href");if(t===null)throw new S;return t}function Y(e,t){e.setAttributeNS(_,"href",t)}var _="http://www.w3.org/1999/xlink",S=class extends Error{constructor(){super("could not find copy icon")}};function a(e){let t=document.querySelector(e);if(t===null)throw new k(`could not find ${e}`);return t}function m(e){let t=document.querySelectorAll(e);if(t===null)throw new k(`could not find all '${e}'`);return Array.from(t)}function D(e){navigator.clipboard.writeText(e).then(null,t=>{throw new I(t)})}var I=class extends Error{constructor(t){super(`clipboard copy rejected: '${t}'`)}},k=class extends Error{constructor(t){super(t)}};function N(){let e;try{e=m(".docs-nav-toggle")}catch{return}e.forEach(t=>{let o=t.querySelector(".toggle-icon use");if(o===null){console.log(`no icon found for toggle: '${t}'`);return}t.addEventListener("click",s=>{s.preventDefault();let r=t.parentElement?.parentElement;if(r==null)return;let l=r.querySelector(".docs-nav-section");l!==null&&(l.classList.toggle("hidden"),J(l)?g(o,"chevron-right","chevron-down"):g(o,"chevron-down","chevron-right"))})})}function J(e){return e.classList.contains("hidden")}function K(){let e,t,o,s;try{e=m(".installer-button"),t=a("#installer-cmd"),o=a("#installer-copy"),s=a("#installer-copy > svg > use")}catch{return}e.forEach(r=>{r.addEventListener("click",l=>{l.preventDefault(),e.forEach(n=>delete n.dataset.active),r.dataset.active="true",t.innerText=$(r.dataset.platform)}),r.dataset.active&&r.dataset.active==="true"&&(t.innerText=$(r.dataset.platform))}),o.addEventListener("click",r=>{r.preventDefault(),D(t.innerText),g(s,"clipboard","check"),setTimeout(()=>g(s,"check","clipboard"),1500)})}function $(e){if(e===void 0)throw new x(e);switch(e){case"macos":return W;case"linux":return W;case"windows":return Z;default:throw new x(e)}}var P=`${globalThis.window.location.protocol}//${globalThis.window.location.host}`,W=`curl -LsSf ${P}/dl/install.sh | sh`,Z=`irm ${P}/dl/install.ps1 | iex`,x=class extends Error{constructor(t){super(`could not determine platform: '${t||"undefined"}'`)}};function q(){m('a[href^="#"]').forEach(e=>{e.addEventListener("click",t=>{if(t.preventDefault(),t.currentTarget===null)return;let o=t.currentTarget.getAttribute("href");if(o===null)return;let r=a(o).getBoundingClientRect().top,l=globalThis.window.scrollY,n=r+l;globalThis.window.scrollTo({top:n,behavior:"smooth"})})})}function O(e,t){let o;return(...s)=>{clearTimeout(o),o=setTimeout(()=>t(...s),e)}}function Q(e){let t=Object.entries(e).filter(([o,s])=>s!==void 0);return Object.fromEntries(t)}function p(e,t={},o={},s=[]){let r=document.createElement(e);Object.assign(r,Q(t)),o.classList&&o.classList.forEach(n=>r.classList.add(n)),o.dataset&&Object.entries(o.dataset).filter(([n,d])=>d!==void 0).forEach(([n,d])=>r.dataset[n]=d);let l=s.map(n=>typeof n=="string"?document.createTextNode(n):n);return r.append(...l),r}function C(){let e=a("#search-button"),t=a("#search-modal"),o=a("#search-modal-close"),s=a("#search-modal-shroud"),r=a("#search-modal-box"),l=a("#search-input");e.addEventListener("click",n=>H(n,t,l)),s.addEventListener("click",n=>H(n,t,l)),o.addEventListener("click",n=>H(n,t,l)),r.addEventListener("click",n=>n.stopPropagation()),document.addEventListener("keydown",n=>{if(n.metaKey===!0&&n.shiftKey===!1&&n.key==="k"){n.preventDefault(),e.click();return}if(F(t))switch(n.key){case"Escape":n.preventDefault(),s.click();return;case"ArrowDown":break;case"ArrowUp":break;default:break}})}function H(e,t,o){e.preventDefault(),t.classList.toggle("hidden"),F(t)&&o.focus()}function F(e){return!e.classList.contains("hidden")}var ee="/search_index.en.json",te=6;function j(){let e=a("#search-input"),t=a("#search-results"),o=a("#search-results-items"),s,r="",l=async function(){return s===void 0&&(s=fetch(ee).then(async function(n){return await elasticlunr.Index.load(await n.json())})),await s};e.addEventListener("keyup",O(150,async function(){let n=e.value.trim();if(n===r||(t.style.display=n===""?"none":"block",o.innerHTML="",r=n,r===""))return;let d=(await l()).search(n,{bool:"AND",fields:{title:{boost:2},body:{boost:1}}});if(d.length===0){t.style.display="none";return}for(let f=0;f<Math.min(d.length,te);++f){let u=ne(d[f],r.split(" "));o.appendChild(u)}}))}function ne(e,t){return p("li",{},{classList:["border-t","border-neutral-300","dark:border-neutral-500"]},[p("div",{},{},[p("a",{href:e.ref},{classList:["block","px-5","py-2","hover:bg-blue-50","dark:hover:bg-blue-500","hover:text-blue-500","dark:hover:text-white","group"]},[p("span",{},{classList:["block","text-base","mb-1","font-medium"]},[e.doc.title]),p("span",{},{classList:["block","text-neutral-500","text-sm","group-hover:text-blue-500"]},[re(e.doc.body,t)])])])])}function re(e,t){let n=t.map(function(c){return elasticlunr.stemmer(c.toLowerCase())}),d=!1,f=0,u=[],A=e.toLowerCase().split(". ");for(let c in A){let i=A[c].split(" "),v=8;for(let B in i){let b=i[B];if(b.length>0){for(let z in n)elasticlunr.stemmer(b).startsWith(n[z])&&(v=40,d=!0);u.push([b,v,f]),v=2}f+=b.length,f+=1}f+=1}if(u.length===0){let c=e,i=p("span",{},{},[]);return i.innerHTML=c,i}let T=[],y=Math.min(u.length,15),E=0;for(let c=0;c<y;c++)E+=u[c][1];T.push(E);for(let c=0;c<u.length-y;c++)E-=u[c][1],E+=u[c+y][1],T.push(E);let w=0;if(d){let c=0;for(let i=T.length-1;i>=0;i--)T[i]>c&&(c=T[i],w=i)}let h=[],L=u[w][2];for(let c=w;c<w+y;c++){let i=u[c];L<i[2]&&(h.push(e.substring(L,i[2])),L=i[2]),i[1]===40&&h.push("<b>"),L=i[2]+i[0].length,h.push(e.substring(i[2],L)),i[1]===40&&h.push("</b>")}h.push("\u2026");let X=h.join(""),R=p("span",{},{},[]);return R.innerHTML=X,R}function G(e){if(e===void 0)throw new M(e);switch(e){case"system":switch(localStorage.removeItem("theme"),se()){case"dark":document.documentElement.classList.add("dark");break;case"light":document.documentElement.classList.remove("dark");break}break;case"light":localStorage.setItem("theme",e),document.documentElement.classList.remove("dark");break;case"dark":localStorage.setItem("theme",e),document.documentElement.classList.add("dark");break;default:throw new M(e)}oe(e)}function oe(e){m(".theme-option").forEach(t=>{t.dataset.theme===e?t.dataset.active="true":delete t.dataset.active})}function se(){return globalThis.window.matchMedia("(prefers-color-scheme: dark)").matches?"dark":"light"}var M=class extends Error{constructor(t){super(`could not determine theme: '${t||"undefined"}'`)}};function U(){m(".theme-option").forEach(e=>{e.addEventListener("click",t=>{t.preventDefault(),G(e.dataset.theme)})})}function ce(){U(),K(),C(),j(),q(),N()}(function(){try{ce()}catch(e){console.error(e)}})();
//# sourceMappingURL=footer.mjs.map
