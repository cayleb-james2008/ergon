import sys, json, pathlib
sys.path.insert(0, '/tmp/checker-layer5/repo/tools')
sys.path.insert(0, '/tmp/checker-layer5/repo/tests/tools')
import test_vinegar_run as T
import vinegar_run as V

t = T.FixtureTests('test_atlas_duplicate_capture_retries_once_and_keeps_the_attempts')
t.setUp()
try:
    fake, ctx = t.atlas()
    def script(f):
        # view-002 first attempt duplicates view-001 (capture #2 -> png 1), retry fresh (capture #3 -> png 2)
        if f.captures == 2: return 1
        if f.captures == 3: return 2
        # view-020 first attempt + both retries duplicate view-019's accepted frame (png 20)
        if f.captures in (21, 22, 23): return 20
        return f.captures
    fake.capture_script = script
    code, result = t.run_fixture(ctx, 'atlas:scout')
    audit_path = pathlib.Path(result['runDir']) / 'capture-attempts.json'
    audit = json.loads(audit_path.read_text()) if audit_path.is_file() else None
    print(json.dumps({
        'exit': code,
        'error': (result.get('error') or '')[:200],
        'result_attempt_entries': len(result.get('captureAttempts', [])),
        'result_phases': [e['phase'] for e in result.get('captureAttempts', [])][:5] + ['...'] + [e['phase'] for e in result.get('captureAttempts', [])][-3:],
        'audit_file_present': audit is not None,
        'audit_entries': len(audit) if audit else None,
        'audit_phases': [e['phase'] for e in audit] if audit else None,
        'consistent_lengths': (len(result.get('captureAttempts', [])) == len(audit)) if audit else None,
        'result_view2_attempts': [a['duplicate'] for a in next((e for e in result.get('captureAttempts', []) if e['phase']=='view-002'), {}).get('attempts', [])],
        'result_view20_attempts': [a['duplicate'] for a in next((e for e in result.get('captureAttempts', []) if e['phase']=='view-020'), {}).get('attempts', [])],
        'audit_last_phase': audit[-1]['phase'] if audit else None,
    }, indent=1))
finally:
    t.doCleanups()
