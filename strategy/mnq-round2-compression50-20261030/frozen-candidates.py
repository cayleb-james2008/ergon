"""Frozen intraday-only candidate definitions; signals consume only closed bars."""
from decimal import Decimal as D

def definitions():
 out=[]
 for market in ('MNQ','MES'):
  stop=120 if market=='MNQ' else 48
  for family,params in (('compression_break',(50,75)),('failed_break',(1,2)),('range_location',(20,30)),('trend_pullback',(4,8))):
   for p in params:out.append({'id':f'{market}-{family}-{p}','market':market,'family':family,'parameter':p,'stop_ticks':stop})
 return out

def signal(candidate,bars):
 """bars[m] = (open,high,low,close,volume); signal at 09:59 ET close."""
 if any(m not in bars for m in range(570,600)):raise ValueError('missing signal bars')
 p=candidate['parameter'];family=candidate['family'];close=bars[599][3];tick=D('.25')
 if family=='compression_break':
  first_hi=max(bars[m][1] for m in range(570,580));first_lo=min(bars[m][2] for m in range(570,580))
  next_hi=max(bars[m][1] for m in range(580,590));next_lo=min(bars[m][2] for m in range(580,590))
  first=first_hi-first_lo;compressed=first>0 and (next_hi-next_lo)*100<=first*p
  side=(int(close>next_hi)-int(close<next_lo)) if compressed else 0
 elif family=='failed_break':
  high=max(bars[m][1] for m in range(570,580));low=min(bars[m][2] for m in range(570,580))
  above=max(bars[m][1] for m in range(580,600))>=high+tick*p and close<high
  below=min(bars[m][2] for m in range(580,600))<=low-tick*p and close>low
  side=int(below)-int(above)
 elif family=='range_location':
  high=max(bars[m][1] for m in range(570,590));low=min(bars[m][2] for m in range(570,590));width=high-low
  side=(int(close<=low+width*D(p)/100)-int(close>=high-width*D(p)/100)) if width>0 else 0
 elif family=='trend_pullback':
  direction=int(bars[584][3]>bars[570][0])-int(bars[584][3]<bars[570][0])
  if direction>0:
   pulled=min(bars[m][2] for m in range(585,600))<=max(bars[m][1] for m in range(570,585))-tick*p
   side=int(pulled and close>bars[598][3])
  elif direction<0:
   pulled=max(bars[m][1] for m in range(585,600))>=min(bars[m][2] for m in range(570,585))+tick*p
   side=-int(pulled and close<bars[598][3])
  else:side=0
 else:raise ValueError('unsupported family')
 return side,599
