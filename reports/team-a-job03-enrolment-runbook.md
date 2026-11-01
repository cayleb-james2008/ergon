# Job 03 — first-employee enrolment runbook (draft)

Prepared by Team A, 2026-10-31. This is the plain-language runbook Cayleb can follow at RHL once the package and host
exist. Draft: final wording waits on the package, but the shape is fixed by the doctrine (apps hosted on one machine per
company, everyone uses a shareable non-localhost URL, nobody installs anything to use an app, the assistant's home is the
company host, per-device enrolment with zero added human friction beyond the device approval).

## Manager pre-step (one person, one minute per employee)

The manager opens the management screen on the company host and:
1. Adds the employee's name and role (the role decides what the assistant knows from day one).
2. Keeps the "approve this device" screen open — the employee's phone/laptop will appear there during enrolment.

That is all the manager does per employee; no accounts to create, no passwords to issue.

## Employee steps (give this section to the employee as-is)

1. **Open the link the company sent you** in your normal browser (Chrome/Safari/Edge on your laptop or phone). You will see
   the company name and a button **"Join this device"**. **Nothing is installed on your computer to use the apps.**
2. **Click "Join this device".** The page shows a short code. Your manager approves the device on their screen; your page
   then says **"This device is approved"**.
3. **Set your sign-in.** Choose a passkey (fingerprint/face, or your device PIN) and save the one-time recovery code the page
   shows you. You will see **"Signed in on this device"**. The recovery code is yours — keep it where you keep your other
   recovery codes.
4. **Meet your assistant.** The chat opens with your one personal assistant, already knowing your role and the company's
   tools. Ask it the first suggested question ("What is on my plate today?"). You will see an answer in plain language.
5. **Add your phone.** On your phone, open the same link, tap **"Add this phone"**, and approve it. You will see the phone
   listed under your approved devices.
6. **Voice (optional).** In any app, press the microphone; the first time, the page offers **"Enable voice"**. After you
   allow it, a small badge reads **"Voice control activated"**. Everything works with typing too — voice is optional.
7. **Everyday use.** Open the link, ask the assistant, use the apps. There is nothing to install, and your assistant keeps
   working when your desktop is off — it lives on the company host.

## Manager extra steps (one page, keep for later)

- **Re-issue a recovery code:** find the employee in the management screen → *Recovery* → issue a new one-time code; the old
  one stops working. (Employee signs in with the new code and sets a new passkey.)
- **Replace a device:** approve the new device, then remove the old one from the same screen. The old device loses access
  immediately.
- **Lost phone:** remove the phone from the device list; the assistant still works from the desk and the laptop.
- **New employee:** repeat the manager pre-step for the person, then hand them the employee steps.

## What NOT to do (and why)

- **Do not send passwords.** There are no passwords to send: the sign-in is a passkey plus a one-time recovery code the
  system shows the employee directly.
- **Do not install the app on each computer.** The apps live on the company host; the URL is the product.
- **Do not use `localhost` or an internal IP** as the link: the product requires the shareable company URL. If a link starts
  with `127.0.0.1` or a raw IP, stop and ask the team — that is the wrong build or the wrong host.
- **Do not share one employee's sign-in.** Devices are approved per person; a shared sign-in is removed and reissued.

## What the team verifies before this runbook is used for real

One complete rehearsal pass on a fresh host (the Job 03 done-bar), a second device reaching the shareable URL, phone
enrolment and the voice/message flow, and the fresh-checker acceptance battery on the final package. Until those exist,
this runbook is a draft, not a promise.
