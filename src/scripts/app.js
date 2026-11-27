import { invoke } from '@tauri-apps/api/core';

const $ = (s) => document.querySelector(s);
const state = { httpd:false, mariadb:false, busy:false, busyAction:null };
let toastTimer;
function toast(message){ const el=$('#toast'); el.textContent=message; el.classList.add('show'); clearTimeout(toastTimer); toastTimer=setTimeout(()=>el.classList.remove('show'),2800); }
function message(text){ $('#message').textContent=text; }
function paint(){
  let count=0;
  for(const name of ['httpd','mariadb']){
    const row=document.querySelector(`[data-service="${name}"]`);
    const pill=$(`#status-${name}`);
    row.classList.toggle('running',state[name]);
    pill.querySelector('span').textContent=state[name]?'Running':'Stopped';
    count+=state[name]?1:0;
  }
  $('#runningCount').textContent=`${count} of 2 running`;
  $('#globalIndicator').classList.toggle('active',count===2);
  const allRunning = count === 2;
  $('#startAll').classList.toggle('primary', !allRunning);
  $('#startAll').classList.toggle('secondary', allRunning);
  $('#stopAll').classList.toggle('primary', allRunning);
  $('#stopAll').classList.toggle('secondary', !allRunning);
  $('#startAll').disabled=state.busy; $('#stopAll').disabled=state.busy;
  $('#startAll').innerHTML=(state.busy && state.busyAction==='start_all')?'Working…':'<span>▶</span> Start All';
  $('#stopAll').innerHTML=(state.busy && state.busyAction==='stop_all')?'Working…':'<span>■</span> Stop All';
}
async function refresh(){
  try{
    const statuses=await invoke('get_service_status');
    state.httpd=statuses.httpd; state.mariadb=statuses.mariadb; paint();
  }catch(e){ message(`Status error: ${e}`); }
}
async function run(action, service){
  if(state.busy)return; state.busy=true; state.busyAction=action; paint(); message('Menjalankan perintah sistem…');
  try{
    const result=await invoke(action, service?{service}:undefined);
    toast(result || 'Perintah berhasil dijalankan'); message(result || 'Perintah selesai');
  }catch(e){ toast(String(e)); message(`Gagal: ${e}`); }
  finally{state.busy=false; state.busyAction=null; await refresh(); paint();}
}
$('#startAll').addEventListener('click',()=>run('start_all'));
$('#stopAll').addEventListener('click',()=>run('stop_all'));
document.querySelectorAll('[data-toggle]').forEach(btn=>btn.addEventListener('click',()=>{
  const service=btn.dataset.toggle; run(state[service]?'stop_service':'start_service',service);
}));
document.querySelectorAll('[data-reload]').forEach(btn=>btn.addEventListener('click',()=>run('reload_service',btn.dataset.reload)));
document.querySelectorAll('[data-action]').forEach(btn=>btn.addEventListener('click',async()=>{
  try{const result=await invoke('quick_action',{action:btn.dataset.action}); if(result)toast(result);}
  catch(e){toast(String(e));}
}));
document.querySelectorAll('[data-config]').forEach(btn=>btn.addEventListener('click',async()=>{
  try{await invoke('open_config',{kind:btn.dataset.config});$('#menuPanel').classList.add('hidden');}
  catch(e){toast(String(e));}
}));
$('#menuButton').addEventListener('click',()=>$('#menuPanel').classList.toggle('hidden'));
document.addEventListener('click',e=>{if(!e.target.closest('#menuButton')&&!e.target.closest('#menuPanel'))$('#menuPanel').classList.add('hidden');});
function setTheme(theme){document.documentElement.dataset.theme=theme;localStorage.setItem('archagon-theme',theme);$('#lightTheme').classList.toggle('active',theme==='light');$('#darkTheme').classList.toggle('active',theme==='dark');}
$('#lightTheme').addEventListener('click',()=>setTheme('light'));
$('#darkTheme').addEventListener('click',()=>setTheme('dark'));
setTheme(localStorage.getItem('archagon-theme')||'light');

let projects=[];
function escapeHtml(value){return String(value).replace(/[&<>"']/g,c=>({'&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot;',"'":'&#39;'}[c]));}
function renderProjects(){
  const query=$('#projectSearch').value.trim().toLowerCase();
  const visible=projects.filter(project=>project.name.toLowerCase().includes(query));
  $('#projectList').innerHTML=visible.length?visible.map(project=>`<article class="project-card"><div class="project-details"><div class="project-title"><strong>${escapeHtml(project.name)}</strong><span class="project-type">${escapeHtml(project.kind)}</span></div><code>${escapeHtml(project.path)}</code></div><div class="project-actions"><button data-project-action="open" data-project="${escapeHtml(project.name)}">↗ Open</button><button data-project-action="terminal" data-project="${escapeHtml(project.name)}">⌘ Terminal</button><button data-project-action="folder" data-project="${escapeHtml(project.name)}">▣ Folder</button></div></article>`).join(''):`<div class="project-empty"><strong>${query?'No matching projects':'No projects found'}</strong><span>${query?'Try another search.':'Create a project or add a folder inside /srv/http.'}</span></div>`;
}
async function refreshProjects(){
  const list=$('#projectList'); list.innerHTML='<div class="project-empty">Scanning /srv/http…</div>';
  try{projects=await invoke('list_projects');renderProjects();}
  catch(e){projects=[];list.innerHTML=`<div class="project-empty"><strong>Could not load projects</strong><span>${escapeHtml(e)}</span></div>`;}
}
$('#projectsToggle').addEventListener('click',()=>{
  const showProjects=$('#projectsView').classList.contains('hidden');
  $('#projectsView').classList.toggle('hidden',!showProjects);$('#dashboardView').classList.toggle('hidden',showProjects);
  $('#projectsToggle').textContent=showProjects?'Dashboard':'Projects';$('#menuPanel').classList.add('hidden');
  if(showProjects)refreshProjects();
});
$('#refreshProjects').addEventListener('click',refreshProjects);
$('#projectSearch').addEventListener('input',renderProjects);
$('#projectList').addEventListener('click',async e=>{
  const button=e.target.closest('[data-project-action]');if(!button)return;
  const command={open:'open_project',terminal:'open_project_terminal',folder:'open_project_folder'}[button.dataset.projectAction];
  try{await invoke(command,{name:button.dataset.project});}
  catch(error){toast(String(error));}
});
$('#newProject').addEventListener('click',()=>{$('#projectModal').classList.remove('hidden');$('#projectName').focus();});
$('#cancelProject').addEventListener('click',()=>$('#projectModal').classList.add('hidden'));
$('#projectForm').addEventListener('submit',async e=>{
  e.preventDefault();
  const form=e.currentTarget;const submit=form.querySelector('[type="submit"]');submit.disabled=true;
  try{await invoke('create_project',{name:$('#projectName').value,kind:$('#projectKind').value});$('#projectModal').classList.add('hidden');form.reset();await refreshProjects();toast('Project created.');}
  catch(error){toast(String(error));}
  finally{submit.disabled=false;}
});

const vhostToggle=$('#vhostToggle');
vhostToggle.checked=localStorage.getItem('archagon-vhosts')==='true';
vhostToggle.addEventListener('change',async()=>{
  localStorage.setItem('archagon-vhosts',String(vhostToggle.checked));
  $('#vhostNote').textContent=vhostToggle.checked?'Memindai folder /srv/http setiap 8 detik. Pastikan IncludeOptional sudah ditambahkan di httpd.conf.':'Auto Virtual Hosts nonaktif.';
  if(vhostToggle.checked) syncVhosts();
});
async function syncVhosts(){
  if(!vhostToggle.checked)return;
  try{const result=await invoke('sync_vhosts'); if(result)message(result);}
  catch(e){toast(`VHost: ${e}`);}
}
if(vhostToggle.checked)$('#vhostNote').textContent='Memindai folder /srv/http setiap 8 detik. Pastikan IncludeOptional sudah ditambahkan di httpd.conf.';
refresh(); setInterval(refresh,2500); setInterval(syncVhosts,8000);
