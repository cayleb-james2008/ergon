#!/usr/bin/env python3
"""Checker-only offline exposed historical walk-forward evaluation; no trading."""
import argparse,csv,hashlib,json,os,platform,sys
from datetime import datetime,timezone
from decimal import Decimal as D
from pathlib import Path
from zoneinfo import ZoneInfo
import candidates,ledger,portfolio
ROOT=Path(__file__).resolve().parent
HIST=ROOT.parent/'trial-factory-frozen-1397'/'artifacts'
NY=ZoneInfo('America/New_York')

def sha(path):
 h=hashlib.sha256()
 with open(path,'rb') as f:
  for b in iter(lambda:f.read(1024*1024),b''):h.update(b)
 return h.hexdigest()

def load_data(path,reference,expected_days):
 result={d:{'MNQ':{},'MES':{}} for d in expected_days}
 with open(path,newline='') as f:
  reader=csv.DictReader(f)
  if reader.fieldnames!=['bar_open_utc','observed_at_utc','market','symbol','instrument_id','open','high','low','close','volume']:raise ValueError('data schema')
  for row in reader:
   t=datetime.fromisoformat(row['bar_open_utc'].replace('Z','+00:00'))
   if t.utcoffset()!=timezone.utc.utcoffset(t) or t.second or t.microsecond:raise ValueError('invalid timestamp')
   local=t.astimezone(NY);day=local.date().isoformat();minute=local.hour*60+local.minute;market=row['market']
   if day not in result or market not in ('MNQ','MES') or minute not in range(570,691):raise ValueError('out of scope data')
   ref=reference['sessions'][day][market]
   if row['symbol']!=ref['symbol'] or row['instrument_id']!=str(ref['instrument_id']):raise ValueError('identity mismatch')
   bars=result[day][market]
   if minute in bars:raise ValueError('duplicate minute')
   o,h,l,c=portfolio._bar([row[x] for x in ('open','high','low','close')]);v=int(row['volume'])
   if v<0 or str(v)!=row['volume']:raise ValueError('invalid volume')
   bars[minute]=(o,h,l,c,v)
 for d in expected_days:
  for market in ('MNQ','MES'):
   if set(result[d][market])!=set(range(570,691)):raise ValueError('missing minutes')
 return result

def session_ranges(days,split):
 train=[d for d in days if split['train_start']<=d<=split['train_end']]
 test=[d for d in days if split['test_start']<=d<=split['test_end']]
 if not train or len(test)<=split['embargo_sessions']:raise ValueError('empty walk-forward segment')
 embargo=test[:split['embargo_sessions']];test=test[split['embargo_sessions']:]
 if train[-1]>=test[0]:raise ValueError('nonchronological split')
 return train,embargo,test

def replay_one(c,days,data,policy):
 sessions=[]
 for day in days:
  b=data[day][c['market']];side,last=candidates.signal(c,b);minute=last+2
  orders={minute:[{'market':c['market'],'qty':1,'side':side,'stop_ticks':c['stop_ticks'],'signal_minute':last}]} if side else {}
  sessions.append({'date':day,'session_flat_minute':690,'orders':orders,'bars':{m:{c['market']:b[m][:4]} for m in range(570,691)}})
 result=portfolio.replay(sessions,policy)
 trades=sum(e['kind']=='entered' for e in result['events']);nets=[D(d['daily_net_usd']) for d in result['days']];total=sum(nets,D(0));equity=D(0);high=D(0);draw=D(0)
 for n in nets:
  equity+=n;high=max(high,equity);draw=max(draw,high-equity)
 return {'trades':trades,'net_usd':str(total),'max_end_of_day_drawdown_usd':str(draw),'drawdown_breached':any(d['drawdown_breached'] for d in result['days']), 'days':result['days'],'events':result['events']}

def select_training(cands,metrics):
 groups={}
 for c in cands:
  if metrics[c['id']]['trades']>0:groups.setdefault((c['market'],c['family']),[]).append(c)
 return {market+'|'+family:sorted(items,key=lambda c:(-D(metrics[c['id']]['net_usd']),c['id']))[0]['id'] for (market,family),items in sorted(groups.items())}

def evaluate(data,manifest):
 days=manifest['sessions'];cs=manifest['candidates'];results=[];pooled={c['id']:{s:{'net':D(0),'trades':0} for s in ('baseline','double_cost')} for c in cs}
 for split in manifest['walk_forward_splits']:
  train,embargo,test=session_ranges(days,split); rows=[];train_metrics={};train_rows={}
  # Complete all training, and freeze selection, before any test signal is evaluated.
  for c in cs:
   cases={}
   for scenario in ('baseline','double_cost'):
    cases[scenario]=replay_one(c,train,data,manifest['cost_scenarios'][scenario])
   train_rows[c['id']]=cases;train_metrics[c['id']]=cases['baseline']
  choices=select_training(cs,train_metrics)
  for c in cs:
   cases={}
   for scenario in ('baseline','double_cost'):
    te=replay_one(c,test,data,manifest['cost_scenarios'][scenario]);cases[scenario]={'train':train_rows[c['id']][scenario],'test':te}
    pooled[c['id']][scenario]['net']+=D(te['net_usd']);pooled[c['id']][scenario]['trades']+=te['trades']
   rows.append({'candidate':c['id'],'scenarios':cases})
  results.append({'split':split['name'],'train_dates':[train[0],train[-1]],'embargo_dates':embargo,'test_dates':[test[0],test[-1]],'train_only_selection':choices,'all_candidates':rows})
 floors=[]
 for c in cs:
  for scenario in ('baseline','double_cost'):
   x=pooled[c['id']][scenario]
   floors.append({'candidate':c['id'],'scenario':scenario,'trades':x['trades'],'net_usd':str(x['net']),'modeled_r3':('NOT_EVALUABLE_NO_TRADES' if x['trades']==0 else 'MODELED_NONNEGATIVE' if x['net']>=0 else 'MODELED_NEGATIVE')})
 return {'status':'EXPOSED_DEVELOPMENT_ONLY','walk_forward':results,'all_candidate_pooled_test_results':floors,'significance':'NOT_EVALUATED','limitations':['Costs and firm policies are hypothetical; fills, spread, latency and commissions not measured.','Historical periods have been previously exposed and coverage-filtered; no blind, forward, profitability, or live readiness claim.','Multiple variants and correlated tests inflate false discoveries; all rows are reported.','Daily-close drawdown is not intraday drawdown. Separate fold accounts reset; pooled net is not a single compounded account.']}

def main():
 a=argparse.ArgumentParser();a.add_argument('--manifest',required=True);a.add_argument('--manifest-sha256',required=True);a.add_argument('--checker-session',required=True);a.add_argument('--ledger',required=True);a.add_argument('--expected-head',required=True);a.add_argument('--output',required=True);x=a.parse_args()
 mp=Path(x.manifest);raw=mp.read_bytes()
 if hashlib.sha256(raw).hexdigest()!=x.manifest_sha256:raise ValueError('manifest mismatch')
 m=json.loads(raw)
 if not x.checker_session or x.checker_session==m['builder_session']:raise ValueError('fresh checker required')
 if m['stage']!='exposed_development' or m['candidates']!=candidates.definitions() or m['sessions']!=sorted(set(m['sessions'])):raise ValueError('scope mismatch')
 if platform.python_version()!=m['runtime']['python_version'] or sha(sys.executable)!=m['runtime']['python_executable_sha256'] or sha('/usr/share/zoneinfo/America/New_York')!=m['runtime']['timezone_sha256']:raise ValueError('runtime changed')
 for f,h in m['file_sha256'].items():
  if sha(ROOT/f)!=h:raise ValueError('code/evidence hash mismatch')
 if sha(HIST/'plan-1397.json')!=m['historical_plan_sha256'] or sha(HIST/'normalized-1397.csv')!=m['input_sha256']:raise ValueError('historical input hash mismatch')
 if Path(x.output).exists():raise ValueError('result path exists')
 event={'kind':'look_claim','stage':'exposed_development','manifest_sha256':x.manifest_sha256,'checker':x.checker_session,'exposure_group':'correlated-equity-index-futures','reuse_policy':'recorded_development_reuse','start':m['sessions'][0],'end':m['sessions'][-1],'candidate_ids':[c['id'] for c in m['candidates']]}
 head=ledger.claim(x.ledger,event,x.expected_head,False)
 try:
  reference=json.loads((HIST/'plan-1397.json').read_bytes());data=load_data(HIST/'normalized-1397.csv',reference,m['sessions'])
  if sha(HIST/'normalized-1397.csv')!=m['input_sha256']:raise ValueError('input changed during read')
  result=evaluate(data,m);result.update(manifest_sha256=x.manifest_sha256,input_sha256=m['input_sha256'],checker=x.checker_session,claim_head=head)
  with open(x.output,'xb') as f:f.write(ledger.canonical(result));f.flush();os.fsync(f.fileno())
  digest=sha(x.output);outhead=ledger.append(x.ledger,{'kind':'result','manifest_sha256':x.manifest_sha256,'result_sha256':digest})
  print(json.dumps({'status':'COMPLETE_EXPOSED_DEVELOPMENT','result_sha256':digest,'ledger_head':outhead}))
 except Exception as exc:
  ledger.append(x.ledger,{'kind':'failure','manifest_sha256':x.manifest_sha256,'error_type':type(exc).__name__,'reason':str(exc)})
  raise
if __name__=='__main__':main()
