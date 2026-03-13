const root=document.getElementById('monitor-app-root');
if(root){
let tab='dashboard',data={feed:[],status:{},stats:{}},search='';
async function refresh(){try{data=await (await fetch('/api/monitor',{cache:'no-store'})).json();render();}catch{}}
function f(n){return new Intl.NumberFormat().format(n??0)}
function render(){
const feed=(data.feed||[]).filter(i=>(i.title||'').toLowerCase().includes(search.toLowerCase()));
root.innerHTML=`<div class=x-hero><p style="letter-spacing:.2em;text-transform:uppercase;color:#fcd34d;font-size:12px">Doginals Explorer</p><h1 style="font-size:32px;margin:6px 0">Modern dApp Control Center</h1></div>
<div class=x-tabs>${['dashboard','inscriptions','protocols','portfolio'].map(t=>`<button class="x-tab ${tab===t?'active':''}" data-tab="${t}">${t}</button>`).join('')}</div>
${tab==='dashboard'?`<div class=x-grid>
<div class=x-card><p>Current Height</p><h3>${data.status.height??'pending'}</h3></div>
<div class=x-card><p>Inscriptions/sec</p><h3>${Number(data.status.inscriptions_per_second??0).toFixed(2)}</h3></div>
<div class=x-card><p>Memory Usage</p><h3>${Math.round((data.stats.memory_usage_bytes??0)/1024/1024)} MB</h3></div>
<div class=x-card><p>Reorg Count</p><h3>${f(data.stats.reorg_count)}</h3></div>
</div><div class="x-card x-feed"><h3>Newly Inscribed Items</h3>${(data.feed||[]).slice(0,10).map(i=>`<a href="${i.link}">${i.kind} · ${i.title}</a>`).join('')}</div>`:''}
${tab==='inscriptions'?`<div class=x-card><input class=x-input placeholder="search inscriptions" value="${search}"><p style="color:#94a3b8">Trait view: kind + height</p>${feed.slice(0,12).map(i=>`<div style="padding:8px 0;border-bottom:1px solid #1f2937"><div style="color:#fbbf24">${i.kind}</div><a href="${i.link}" style="color:#fff">${i.title}</a><div style="color:#94a3b8">${i.subtitle}</div></div>`).join('')}</div>`:''}
${tab==='protocols'?`<div class=x-grid>${[['DRC-20',data.status.inscriptions],['DMP',data.status.dmp],['DogeLotto',data.status.dogelotto],['DogeSpells',data.status.dogespells],['Dogemap',data.status.dogemaps]].map(([l,v])=>`<div class=x-card><p>${l}</p><h3>${f(v)}</h3></div>`).join('')}</div>`:''}
${tab==='portfolio'?`<div class=x-card><h3>My Trades / Portfolio</h3><p>Live Kabosu API source: <code>https://api.kabosu.io</code></p><p style="color:#94a3b8">Connect your wallet integrations to view balances and live fills.</p></div>`:''}`;
root.querySelectorAll('[data-tab]').forEach(b=>b.onclick=()=>{tab=b.dataset.tab;render();});
const input=root.querySelector('.x-input'); if(input) input.oninput=(e)=>{search=e.target.value;render();};
}
refresh();setInterval(refresh,5000);
}
