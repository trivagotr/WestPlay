/** Replay analysis only: never attaches to a process or treats caller metadata as authenticated evidence. */
import {readFileSync,statSync} from 'node:fs';
import {resolve} from 'node:path';
import {pathToFileURL} from 'node:url';
const bad=()=>{throw Error('invalid-observation');};
const point=p=>p&&Number.isFinite(p.x)&&Number.isFinite(p.z)&&Math.abs(p.x)<17000000&&Math.abs(p.z)<17000000;
const distance=(a,b)=>Math.hypot(a.x-b.x,a.z-b.z);
export function analyzeObservation(capture){
 if(!capture||capture.schema!=='westplay-convoy-observation-redacted-v1'||!Array.isArray(capture.samples)||capture.samples.length<1||capture.samples.length>512)bad();
 const samples=capture.samples;let previous=-1;
 for(const s of samples){
  if(!Number.isSafeInteger(s.tMs)||s.tMs<0||s.tMs<=previous||s.tMs>120000||!Number.isFinite(s.readMs)||s.readMs<0||s.readMs>1000||!Array.isArray(s.vehicles)||s.vehicles.length>128||(s.own!==null&&!point(s.own)))bad();
  previous=s.tMs;const ids=new Set();
  for(const v of s.vehicles){if(!point(v)||typeof v.id!=='string'||!v.id.match(/^[a-z0-9_-]{1,48}$/i)||ids.has(v.id))bad();ids.add(v.id);}
 }
 const ownIds=new Set();
 const frames=samples.map(s=>{
  if(!s.own)return {tMs:s.tMs,reason:'own-telemetry-unavailable',remoteCandidates:[]};
  const matches=s.vehicles.filter(v=>distance(v,s.own)<=0.5);
  if(matches.length!==1)return {tMs:s.tMs,reason:matches.length?'ambiguous-own-vehicle':'own-vehicle-unmatched',remoteCandidates:[]};
  ownIds.add(matches[0].id);
  return {tMs:s.tMs,reason:null,remoteCandidates:s.vehicles.filter(v=>v.id!==matches[0].id).map(v=>({id:v.id,x:v.x,z:v.z}))};
 });
 if(ownIds.size!==1)throw Error('invalid-observation-owner');
 const tracks=new Map();
 for(const s of samples)for(const v of s.vehicles){if(!tracks.has(v.id))tracks.set(v.id,[]);tracks.get(v.id).push({t:s.tMs,...v});}
 const actors=[...tracks].map(([id,points])=>({id,role:ownIds.has(id)?'local':'peer-candidate',samples:points.length,maxDisplacement:Math.max(...points.map(p=>distance(p,points[0]))),netDisplacement:distance(points.at(-1),points[0])}));
 let sourceRemoteFrames=0,gapStart=null,gapCount=0,longestMissingRunSamples=0,longestSourceAbsenceMs=0;
 for(const s of samples){
  const present=s.vehicles.some(v=>!ownIds.has(v.id));
  if(present){sourceRemoteFrames++;if(gapStart!==null){longestSourceAbsenceMs=Math.max(longestSourceAbsenceMs,s.tMs-gapStart);gapStart=null;gapCount=0;}}
  else{if(gapStart===null)gapStart=s.tMs;gapCount++;longestMissingRunSamples=Math.max(longestMissingRunSamples,gapCount);}
 }
 if(gapStart!==null)longestSourceAbsenceMs=Math.max(longestSourceAbsenceMs,samples.at(-1).tMs-gapStart);
 const timings=samples.map(s=>s.readMs).sort((a,b)=>a-b);
 return {offlineReplay:true,completeRosterVerified:false,frames,actors,metrics:{frames:samples.length,durationMs:samples.at(-1).tMs-samples[0].tMs,sourceRemoteFrames,missingSourceFrames:samples.length-sourceRemoteFrames,renderableRemoteFrames:frames.filter(f=>f.remoteCandidates.length).length,ownMatchedFrames:frames.filter(f=>f.reason===null).length,localMaxDisplacement:Math.max(0,...actors.filter(a=>a.role==='local').map(a=>a.maxDisplacement)),remoteMaxDisplacement:Math.max(0,...actors.filter(a=>a.role!=='local').map(a=>a.maxDisplacement)),longestMissingRunSamples,longestSourceAbsenceMs,readStageMedianMs:timings[Math.floor(timings.length/2)],readStageMaxMs:timings.at(-1)}};
}
if(process.argv[1]&&import.meta.url===pathToFileURL(resolve(process.argv[1])).href){
 try{if(process.argv.length!==3||statSync(process.argv[2]).size>1024*1024)bad();const result=analyzeObservation(JSON.parse(readFileSync(process.argv[2],'utf8').replace(/^\uFEFF/,'')));console.log(JSON.stringify({offlineReplay:true,completeRosterVerified:false,metrics:result.metrics,actors:result.actors},null,2));}
 catch{console.error('invalid-observation');process.exitCode=1;}
}
