"""Offline futures book risk replay. Synthetic/assumption-bound; no routing or accounts."""
from decimal import Decimal as D
from datetime import date as Date

TICK=D('.25'); PV={'MNQ':D('2'), 'MES':D('5')}


def dec(value):
    x=D(str(value))
    if not x.is_finite(): raise ValueError('nonfinite value')
    return x


def validate_policy(p):
    if p.get('status') not in ('assumed_diagnostic','independently_verified'):
        raise ValueError('policy status must be explicit')
    if p.get('timezone')!='America/New_York': raise ValueError('ET timezone required')
    for field in ('starting_cash','starting_high_water','daily_loss_usd','daily_profit_usd','drawdown_usd','commission_roundtrip'):
        dec(p[field])
    if dec(p['daily_loss_usd'])<=0 or dec(p['daily_profit_usd'])<=0 or dec(p['drawdown_usd'])<=0 or dec(p['commission_roundtrip'])<0:
        raise ValueError('invalid risk/cost')
    if dec(p['daily_loss_usd'])>400 or dec(p['daily_profit_usd'])>900:
        raise ValueError('desk daily rails cannot be loosened')
    if p.get('drawdown_mode') not in ('static','eod_trailing','intraday_trailing'): raise ValueError('unsupported drawdown')
    if not isinstance(p.get('floor_cap'), (str,type(None))): raise ValueError('floor cap required, use None for no cap')
    if p['floor_cap'] is not None: dec(p['floor_cap'])
    if type(p.get('flat_minute')) is not int or not 0<p['flat_minute']<=965: raise ValueError('invalid flat time')
    if type(p.get('cooldown_minutes')) is not int or p['cooldown_minutes']<15: raise ValueError('cooldown shorter than desk rail')
    if type(p.get('slip_ticks')) is not int or p['slip_ticks']<1: raise ValueError('slippage must be positive')
    if p.get('opposite_correlated_positions') not in ('reject','allow_assumed'): raise ValueError('hedging policy missing')
    if type(p.get('total_micro_cap')) is not int or p['total_micro_cap']<1: raise ValueError('total cap missing')
    for market,cap in p.get('market_caps',{}).items():
        if market not in PV or type(cap) is not int or cap<1: raise ValueError('bad market cap')
    if not p.get('market_caps'): raise ValueError('market caps missing')
    if dec(p['starting_high_water']) < dec(p['starting_cash']): raise ValueError('starting high-water below cash')
    if p['status']=='independently_verified' and not p.get('verification_receipt_sha256'):
        raise ValueError('verification receipt required')


def _bar(bar):
    if not isinstance(bar,(tuple,list)) or len(bar)!=4: raise ValueError('bad bar')
    o,h,l,c=map(dec,bar)
    if not all(x>0 and x%TICK==0 for x in (o,h,l,c)) or not (l<=o<=h and l<=c<=h): raise ValueError('bad OHLC')
    return o,h,l,c


def _floor(high,p):
    f=high-dec(p['drawdown_usd'])
    return min(f,dec(p['floor_cap'])) if p['floor_cap'] is not None else f


def replay(sessions,p):
    """Assumption-bound replay. Every session supplies ET minute bars and predecided orders.

    Multi-market intrabar extremes are combined pessimistically, although individual
    extremes need not be synchronous; a breach is a risk bound, not actual execution.
    No position may survive the predeclared daily flat time.
    """
    validate_policy(p)
    cash=dec(p['starting_cash']); high=dec(p['starting_high_water']); floor=_floor(high,p)
    sl=dec(p['slip_ticks'])*TICK; comm=dec(p['commission_roundtrip'])
    events=[]; breached=False; last_date=''; day_results=[]
    if cash<=floor: raise ValueError('starting account already at drawdown floor')
    for day in sessions:
        date=day['date']
        if Date.fromisoformat(date).isoformat()!=date: raise ValueError('invalid session date')
        if date<=last_date: raise ValueError('sessions must be strictly ordered')
        last_date=date
        bars=day['bars']; orders=day['orders']; flat=min(p['flat_minute'],day['session_flat_minute'])
        if not bars or flat not in bars: raise ValueError('missing flat-time bar')
        minutes=sorted(bars)
        if any(type(m) is not int for m in minutes) or any(b-a!=1 for a,b in zip(minutes,minutes[1:])):
            raise ValueError('missing or unordered minute bars')
        if minutes[-1]!=flat: raise ValueError('session must end at its flat-time bar')
        if any(type(m) is not int or m not in bars for m in orders): raise ValueError('order outside observed minutes')
        pos=[]; day_start=cash; halted=breached; pending_profit=False; last_loss=None
        def values(market,minute):
            try: return _bar(bars[minute][market])
            except KeyError: raise ValueError('missing position-market bar') from None
        def marks(minute,kind):
            result=cash
            for q in pos:
                o,h,l,c=values(q['market'],minute)
                raw={'open':o,'close':c,'best':h if q['side']==1 else l,'worst':l if q['side']==1 else h}[kind]
                px=raw-q['side']*sl
                result+=(px-q['entry'])*q['side']*PV[q['market']]*q['qty']
            return result
        def record_close(q,px,reason,minute):
            nonlocal cash,last_loss
            pnl=(px-q['entry'])*q['side']*PV[q['market']]*q['qty'];cash+=pnl
            net=pnl-comm*q['qty']
            if net<0:last_loss=minute
            events.append({'date':date,'minute':minute,'kind':reason,'trade_id':q['trade_id'],
                'market':q['market'],'side':q['side'],'qty':q['qty'],'signal_minute':q['signal_minute'],
                'entry_minute':q['entry_minute'],'entry_price':str(q['entry']),'stop_price':str(q['stop']),
                'exit_price':str(px),'commission_roundtrip_total':str(comm*q['qty']),'net_usd':str(net)})
        def liquidate(minute,kind,reason):
            for q in pos:
                o,h,l,c=values(q['market'],minute)
                raw={'open':o,'worst':l if q['side']==1 else h}[kind]
                record_close(q,raw-q['side']*sl,reason,minute)
            pos.clear()
        def opening_stops(minute):
            for q in pos[:]:
                o=values(q['market'],minute)[0]
                if (q['side']==1 and o<=q['stop']) or (q['side']==-1 and o>=q['stop']):
                    record_close(q,o-q['side']*sl,'protective_stop',minute)
                    pos.remove(q)
        for minute in minutes:
            # The opening print precedes the bar's unknown high/low path.
            if pos:
                open_equity=marks(minute,'open')
                if p['drawdown_mode']=='intraday_trailing':
                    high=max(high,open_equity); floor=max(floor,_floor(high,p))
                if open_equity<=floor:
                    liquidate(minute,'open','drawdown_breach'); breached=True; halted=True
                elif open_equity-day_start<=-dec(p['daily_loss_usd']):
                    liquidate(minute,'open','daily_loss_stop'); halted=True
                elif open_equity-day_start>=dec(p['daily_profit_usd']):
                    liquidate(minute,'open','profit_stop'); halted=True; pending_profit=False
            if pending_profit and pos:
                liquidate(minute,'open','profit_stop'); pending_profit=False; halted=True
            if pos: opening_stops(minute)
            opening_equity=marks(minute,'open') if pos else cash
            if opening_equity<=floor:
                if pos: liquidate(minute,'open','drawdown_breach')
                breached=True; halted=True
            elif opening_equity-day_start<=-dec(p['daily_loss_usd']):
                if pos: liquidate(minute,'open','daily_loss_stop')
                halted=True
            if minute==flat:
                if pos: liquidate(minute,'open','flat')
                halted=True
            for order in orders.get(minute,[]):
                market=order['market']; qty=order['qty']; side=order['side']; stop_ticks=order['stop_ticks']; signal=order['signal_minute']
                if market not in p['market_caps'] or type(qty) is not int or qty<1 or type(side) is not int or side not in (-1,1) or type(stop_ticks) is not int or stop_ticks<1:
                    raise ValueError('invalid order')
                if type(signal) is not int or minute<signal+2: raise ValueError('order uses incomplete or future signal')
                if halted or minute>=flat or (last_loss is not None and minute-last_loss<p['cooldown_minutes']):
                    events.append({'date':date,'minute':minute,'kind':'entry_refused'}); continue
                if sum(q['qty'] for q in pos)+qty>p['total_micro_cap'] or sum(q['qty'] for q in pos if q['market']==market)+qty>p['market_caps'][market]:
                    events.append({'date':date,'minute':minute,'kind':'cap_refused'}); continue
                if p['opposite_correlated_positions']=='reject' and any(q['side']!=side for q in pos):
                    events.append({'date':date,'minute':minute,'kind':'hedge_refused'}); continue
                entry=values(market,minute)[0]+side*sl; stop=entry-side*TICK*stop_ticks
                # Existing and new stops are summed to preserve a book-level loss budget.
                stop_risk=sum((q['entry']-q['stop'])*q['side']*PV[q['market']]*q['qty']+sl*PV[q['market']]*q['qty'] for q in pos)
                stop_risk+=TICK*stop_ticks*PV[market]*qty+sl*PV[market]*qty+comm*qty
                if cash-day_start-stop_risk<=-dec(p['daily_loss_usd']) or cash-stop_risk<=floor:
                    events.append({'date':date,'minute':minute,'kind':'risk_refused'}); continue
                cash-=comm*qty
                trade_id=f'{date}-{minute}-{len(events)}'
                pos.append({'market':market,'qty':qty,'side':side,'entry':entry,'stop':stop,
                            'signal_minute':signal,'entry_minute':minute,'trade_id':trade_id})
                events.append({'date':date,'minute':minute,'kind':'entered','trade_id':trade_id,'market':market,
                               'qty':qty,'side':side,'signal_minute':signal,'entry_price':str(entry),
                               'stop_price':str(stop),'commission_roundtrip_total':str(comm*qty)})
            if pos: opening_stops(minute)
            opening_equity=marks(minute,'open') if pos else cash
            if opening_equity<=floor:
                if pos: liquidate(minute,'open','drawdown_breach')
                breached=True; halted=True
            elif opening_equity-day_start<=-dec(p['daily_loss_usd']):
                if pos: liquidate(minute,'open','daily_loss_stop')
                halted=True
            if not pos: continue
            if p['drawdown_mode']=='intraday_trailing':
                high=max(high,marks(minute,'best')); floor=max(floor,_floor(high,p))
            low=marks(minute,'worst')
            if low<=floor:
                liquidate(minute,'worst','drawdown_breach'); breached=True; halted=True; continue
            if low-day_start<=-dec(p['daily_loss_usd']):
                liquidate(minute,'worst','daily_loss_stop'); halted=True; continue
            for q in pos[:]:
                o,h,l,c=values(q['market'],minute)
                hit=l<=q['stop'] if q['side']==1 else h>=q['stop']
                if hit:
                    raw=min(o,q['stop']) if q['side']==1 else max(o,q['stop'])
                    record_close(q,raw-q['side']*sl,'protective_stop',minute)
                    pos.remove(q)
            post_stop_low=marks(minute,'worst') if pos else cash
            if post_stop_low<=floor:
                if pos: liquidate(minute,'worst','drawdown_breach')
                breached=True; halted=True
            elif post_stop_low-day_start<=-dec(p['daily_loss_usd']):
                if pos: liquidate(minute,'worst','daily_loss_stop')
                halted=True
            if pos and marks(minute,'close')-day_start>=dec(p['daily_profit_usd']): pending_profit=True
        if pos: raise ValueError('position survived flat')
        if p['drawdown_mode']=='eod_trailing':
            high=max(high,cash); floor=max(floor,_floor(high,p))
        day_results.append({'date':date,'cash':str(cash),'daily_net_usd':str(cash-day_start),'floor':str(floor),'drawdown_breached':breached})
    return {'status':'assumption_bound_offline_replay','policy_status':p['status'],
            'limitations':['Intrabar extremes are a pessimistic cross-market bound, not synchronous fills.',
                           'No partial fills, margin, exchange halts, payout, account rules or slippage distribution.'],
            'days':day_results,'events':events}
