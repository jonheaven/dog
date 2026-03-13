import React, { useEffect, useMemo, useState } from 'react';
import { createRoot } from 'react-dom/client';
import { Card, cn } from './components/ui';
import './styles.css';

type Monitor = Awaited<ReturnType<typeof getMonitor>>;

async function getMonitor() {
  const res = await fetch('/api/monitor', { cache: 'no-store' });
  if (!res.ok) throw new Error('monitor unavailable');
  return res.json();
}

function App() {
  const [tab, setTab] = useState('dashboard');
  const [data, setData] = useState<Monitor | null>(null);
  const [search, setSearch] = useState('');
  const [page, setPage] = useState(1);
  const [portfolio, setPortfolio] = useState<any>(null);

  useEffect(() => {
    const pull = async () => setData(await getMonitor());
    pull().catch(() => undefined);
    const id = setInterval(() => pull().catch(() => undefined), 5000);
    return () => clearInterval(id);
  }, []);

  useEffect(() => {
    fetch(`${(window as any).KABOSU_API_BASE ?? 'https://api.kabosu.io'}/portfolio`).then(r => r.json()).then(setPortfolio).catch(() => setPortfolio({ error: 'Kabosu API unavailable' }));
  }, []);

  const feed = useMemo(() => (data?.feed ?? []).filter((i: any) => i.title.toLowerCase().includes(search.toLowerCase())), [data, search]);
  const pageItems = feed.slice((page - 1) * 8, page * 8);

  return (
    <div className='mx-auto max-w-7xl space-y-6 px-4 py-6'>
      <div className='rounded-2xl border border-amber-400/20 bg-gradient-to-br from-slate-950 to-slate-900 p-6'>
        <p className='text-xs uppercase tracking-[0.24em] text-amber-300'>Doginals Explorer</p>
        <h1 className='mt-2 text-3xl font-semibold'>Modern dApp Control Center</h1>
      </div>
      <div className='flex gap-2 overflow-auto'>{['dashboard', 'inscriptions', 'protocols', 'portfolio'].map(t => <button key={t} onClick={() => setTab(t)} className={cn('rounded-lg px-3 py-2 text-sm capitalize', tab === t ? 'bg-amber-400 text-black' : 'bg-slate-800')}>{t}</button>)}</div>

      {tab === 'dashboard' && data && <div className='grid gap-4 md:grid-cols-4'>
        <Card><p>Current Height</p><h3 className='text-2xl font-bold'>{data.status.height ?? 'pending'}</h3></Card>
        <Card><p>Inscriptions/sec</p><h3 className='text-2xl font-bold'>{Number(data.status.inscriptions_per_second).toFixed(2)}</h3></Card>
        <Card><p>Memory Usage</p><h3 className='text-2xl font-bold'>{Math.round(data.stats.memory_usage_bytes / 1024 / 1024)}MB</h3></Card>
        <Card><p>Reorg Count</p><h3 className='text-2xl font-bold'>{data.stats.reorg_count}</h3></Card>
        <Card className='md:col-span-4'><p className='mb-3'>Newly Inscribed Feed</p><div className='space-y-2'>{(data.feed ?? []).slice(0, 10).map((item: any) => <a key={`${item.title}-${item.timestamp}`} href={item.link} className='block rounded-lg border border-slate-800 p-3 hover:bg-slate-800/60'>{item.kind} · {item.title}</a>)}</div></Card>
      </div>}

      {tab === 'inscriptions' && <Card><div className='mb-4 flex items-center justify-between gap-3'><input value={search} onChange={(e) => setSearch(e.target.value)} placeholder='Search inscriptions...' className='w-full rounded-lg border border-slate-700 bg-slate-900 px-3 py-2' /><span className='text-sm text-slate-400'>Trait view: Kind / Height</span></div><div className='space-y-2'>{pageItems.map((item: any) => <div key={`${item.title}-${item.timestamp}`} className='rounded-lg border border-slate-800 p-3'><div className='text-xs text-amber-300'>{item.kind}</div><a className='font-semibold' href={item.link}>{item.title}</a><p className='text-sm text-slate-400'>{item.subtitle}</p></div>)}</div><div className='mt-4 flex gap-2'><button onClick={() => setPage(Math.max(1, page - 1))} className='rounded bg-slate-800 px-3 py-1'>Prev</button><button onClick={() => setPage(page + 1)} className='rounded bg-slate-800 px-3 py-1'>Next</button></div></Card>}

      {tab === 'protocols' && data && <div className='grid gap-4 md:grid-cols-2 lg:grid-cols-3'>{[['DRC-20', data.status.inscriptions], ['DMP Listings', data.status.dmp], ['DogeLotto', data.status.dogelotto], ['DogeSpells', data.status.dogespells], ['Dogemap', data.status.dogemaps]].map(([label, value]) => <Card key={label as string}><p className='text-sm text-slate-400'>{label}</p><p className='text-2xl font-bold'>{value as any}</p></Card>)}</div>}

      {tab === 'portfolio' && <Card><h3 className='mb-2 text-lg font-semibold'>My Trades / Portfolio (Kabosu API)</h3><pre className='overflow-auto text-xs text-slate-300'>{JSON.stringify(portfolio, null, 2)}</pre></Card>}
    </div>
  );
}

const root = document.getElementById('monitor-app-root');
if (root) createRoot(root).render(<React.StrictMode><App /></React.StrictMode>);
