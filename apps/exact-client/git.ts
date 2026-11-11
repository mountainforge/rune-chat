export type Execute = (program: string, args: string[]) => Promise<string>;
export type Commit = { oid: string; author: string; date: string; subject: string; parents: string[]; refs: string; graph: string; graphWidth: number; initials:string; avatar:string; labels:{label:string;kind:number;color:string}[] };
export type ChangedFile = { status: string; path: string; oldPath: string; additions:number; deletions:number; binary:boolean; name:string; directory:string };
const base = ['--no-pager', '--literal-pathspecs', '--no-optional-locks', '-c', 'color.ui=false', '-C'];
export const git = (run: Execute, repo: string, args: string[]) => run('path:git', [...base, repo, ...args]);
const initials=(name:string)=>name.trim().split(/\s+/).slice(0,2).map(s=>s[0]??'').join('').toUpperCase();
const avatar=(name:string)=>{const palette=['#f2c6a0','#a9c7ff','#c3a9ef','#b5dfc4','#efc0cc','#c7d5da'];let n=0;for(const c of name)n=((n*31)+c.charCodeAt(0))>>>0;return palette[n%palette.length];};
const colors = ['#9a5a00','#06748f','#6a3fc7','#23752f','#ae2016','#2c55c2'];
// Pending parent IDs define lanes; labels never determine connectivity.
export function graph(commits: Commit[]) {
  let active: string[] = []; let width = 1;
  const rows = commits.map(c => {
    const incoming = active.includes(c.oid);
    if (!incoming) active.push(c.oid);
    const lane = active.indexOf(c.oid), before = [...active]; active.splice(lane,1);
    c.parents.forEach((p,i) => { if (!active.includes(p)) active.splice(Math.min(lane+i,active.length),0,p); });
    const x = (n: number) => 14+n*18;
    const line = (from: number,to: number,a: number,b: number,color: number) => `<path d="M${x(from)} ${a} C${x(from)} ${a+(b-a)*.5},${x(to)} ${b-(b-a)*.5},${x(to)} ${b}" fill="none" stroke="${colors[color%6]}" stroke-width="1.8"/>`;
    let svg = before.map((id,i) => id === c.oid ? '' : line(i,active.indexOf(id),0,30,i)).join('');
    if (incoming) svg += line(lane,lane,0,15,lane);
    c.parents.forEach(p => { const to=active.indexOf(p); svg += line(lane,to,15,30,to); });
    svg += `<circle cx="${x(lane)}" cy="15" r="4" fill="${colors[lane%6]}"/>`;
    width=Math.max(width,before.length,active.length); c.labels=c.refs.split(", ").filter(Boolean).map(r=>({label:r.startsWith("HEAD -> ")?r.slice(8):r.startsWith("tag: ")?r.slice(5):r,kind:r.startsWith("HEAD -> ")?0:r.startsWith("tag: ")?3:r.startsWith("origin/")?2:1,color:colors[lane%6]}));return svg;
  });
  const graphWidth = Math.max(76,width*18+28);
  return commits.map((c,i) => ({...c,graphWidth,graph:'data:image/svg+xml,'+encodeURIComponent(`<svg xmlns="http://www.w3.org/2000/svg" width="${graphWidth}" height="30" viewBox="0 0 ${graphWidth} 30">${rows[i]}</svg>`)}));
}
export async function history(run: Execute, repo: string, limit: number) {
  if (!repo) return { commits: [] as Commit[], head:'All branches',more: false, error: '' };
  try {
    await git(run,repo,['rev-parse','--absolute-git-dir']);
    let head=true;try {await git(run,repo,['rev-parse','--verify','HEAD']);}catch{head=false;}
    const raw=await git(run,repo,['log','--all',...(head?['HEAD']:[]),'--topo-order','--decorate=short','-z','--format=%H%x00%an%x00%aI%x00%s%x00%P%x00%D',`--max-count=${limit+1}`]);
    const f=raw.split('\0');if(f.at(-1)==='')f.pop();
    if(f.length%6!==0)throw new Error('Unexpected Git log output');
    const commits: Commit[]=[];
    for(let i=0;i<f.length;i+=6)commits.push({oid:f[i],author:f[i+1],date:f[i+2],subject:f[i+3],parents:f[i+4].split(' ').filter(Boolean),refs:f[i+5],graph:'',graphWidth:94,initials:initials(f[i+1]),avatar:avatar(f[i+1]),labels:[]});
    return {commits:graph(commits.slice(0,limit)),head:commits.flatMap(c=>c.refs.split(', ')).find(r=>r.startsWith('HEAD -> '))?.slice(8)??'All branches',more:commits.length>limit,error:''};
  }catch(e){return {commits:[] as Commit[],head:'All branches',more:false,error:String(e)};}
}
export async function detail(run: Execute,repo: string,oid: string) {
  const blank={oid:'',author:'',email:'',date:'',parents:'',message:'',additions:0,deletions:0,initials:'',avatar:'#f2c6a0',files:[] as ChangedFile[],error:''};
  if(!repo||!oid)return blank;
  try {
    if(!/^(?:[a-fA-F0-9]{40}|[a-fA-F0-9]{64})$/.test(oid))throw new Error('Invalid commit ID');
    const raw=await git(run,repo,['show','--no-patch','--format=%H%x00%an%x00%ae%x00%aI%x00%P%x00%B',oid,'--']);
    const f=raw.split('\0');if(f.length!==6)throw new Error('Unexpected Git commit output');
    const parent=f[4].split(' ').filter(Boolean)[0];
    const names=await git(run,repo,['diff-tree','--root','--no-commit-id','-r','-z','--name-status','--find-renames',...(parent?[parent]:[]),oid,'--']);
    const fields=names.split('\0');if(fields.at(-1)==='')fields.pop();const files:ChangedFile[]=[];
    for(let i=0;i<fields.length;){const status=fields[i++],renamed=/^[RC]/.test(status),needed=renamed?2:1;
      if(i+needed>fields.length)throw new Error('Unexpected Git file list');
      const path=fields[i+needed-1],slash=path.lastIndexOf('/');files.push({status,path,oldPath:renamed?fields[i]:'',additions:0,deletions:0,binary:false,name:path.slice(slash+1),directory:slash<0?'Repository root':path.slice(0,slash+1)});i+=needed;
    }
    const statRaw=await git(run,repo,['diff-tree','--root','--no-commit-id','-r','-z','--numstat','--find-renames',...(parent?[parent]:[]),oid,'--']);
    const stats=statRaw.split('\0');if(stats.at(-1)==='')stats.pop();
    for(let i=0;i<stats.length;){const record=stats[i++],a=record.indexOf('\t'),b=record.indexOf('\t',a+1);
      if(a<0||b<0)throw new Error('Unexpected Git statistics');
      const add=record.slice(0,a),del=record.slice(a+1,b);let path=record.slice(b+1);
      if(!path){if(i+2>stats.length)throw new Error('Unexpected Git rename statistics');path=stats[i+1];i+=2;}
      const file=files.find(f=>f.path===path);if(file){file.binary=add==='-';file.additions=Number(add)||0;file.deletions=Number(del)||0;}
    }
    return {oid:f[0],author:f[1],email:f[2],date:f[3],parents:f[4],message:f[5].trimEnd(),initials:initials(f[1]),avatar:avatar(f[1]),additions:files.reduce((n,f)=>n+f.additions,0),deletions:files.reduce((n,f)=>n+f.deletions,0),files,error:''};
  }catch(e){return {...blank,error:String(e)};}
}
export type PatchLine={id:string;text:string;old:string;next:string;kind:number};
export type PatchPair={id:string;left:PatchLine;right:PatchLine;full:boolean};
const blankLine:PatchLine={id:'',text:'',old:'',next:'',kind:5};
export function patchLines(text:string,scope:string):PatchLine[]{
  let old=0,next=0,meta=0,inHunk=false;const hasHunks=text.split('\n').some(s=>s.startsWith('@@ '));const result:PatchLine[]=[];
  for(const line of text.split('\n')){
    const h=/^@@ -(\d+)(?:,\d+)? \+(\d+)(?:,\d+)? @@/.exec(line);
    if(h){old=Number(h[1]);next=Number(h[2]);inHunk=true;result.push({id:scope+':h'+meta++,text:line,old:'',next:'',kind:3});continue;}
    if(!inHunk&&hasHunks)continue;
    const kind=inHunk?(line.startsWith('+')?1:line.startsWith('-')?2:line.startsWith(' ')?0:4):4;
    result.push({id:scope+':'+meta++,text:kind<3?line.slice(1):line,old:kind===0||kind===2?String(old++):'',next:kind===0||kind===1?String(next++):'',kind});
  }return result;
}
export function patchPairs(lines:PatchLine[]):PatchPair[]{
  const result:PatchPair[]=[];let i=0;
  while(i<lines.length){const line=lines[i];
    if(line.kind===1||line.kind===2){const left:PatchLine[]=[],right:PatchLine[]=[];
      while(i<lines.length&&(lines[i].kind===1||lines[i].kind===2)){(lines[i].kind===2?left:right).push(lines[i]);i++;}
      for(let n=0;n<Math.max(left.length,right.length);n++)result.push({id:(left[n]??right[n]).id,left:left[n]??blankLine,right:right[n]??blankLine,full:false});
    }else{result.push({id:line.id,left:line,right:line.kind===0?line:blankLine,full:line.kind>=3});i++;}
  }return result;
}
export async function filePatch(run:Execute,repo:string,oid:string,path:string,oldPath:string) {
  if(!repo||!oid||!path)return {text:'',lines:[] as PatchLine[],pairs:[] as PatchPair[],error:''};
  try {
    if(!/^(?:[a-fA-F0-9]{40}|[a-fA-F0-9]{64})$/.test(oid))throw new Error('Invalid commit ID');
    let text=await git(run,repo,['show','--format=','--patch','--root','--diff-merges=first-parent','--find-renames','--no-ext-diff','--no-textconv',oid,'--',path,...(oldPath?[oldPath]:[])]);
    if(text.length>2_000_000)text=text.slice(0,2_000_000)+'\n[Display truncated at 2 MB]';
    const lines=patchLines(text,oid+'|'+path);return {text,lines,pairs:patchPairs(lines),error:''};
  }catch(e){return {text:'',lines:[] as PatchLine[],pairs:[] as PatchPair[],error:String(e)};}
}
