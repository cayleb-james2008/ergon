# Job 03 — one-screen host purchase card for Cayleb

Prepared by Team A, 2026-10-31. **This is the smallest possible act: one purchase, ~2 minutes, ~€8/month.** Nothing else is
needed from you for this step; the team does the rest once the host exists. Do not buy anything else, and do not paste a
password or key anywhere except where this card says.

## What you are buying

A **single small cloud server** that will host one company's Ergon apps — RHL's own, not shared with anyone else. It is the
"company cloud host" the product design requires: the apps and each employee's assistant live there, so they work even
when a desktop is off. One server per company, per the locked hosting ruling.

## Where and what

| Item | Exact choice |
|---|---|
| Provider | **Hetzner Cloud** (`console.hetzner.cloud`) — the ruled standard |
| Location | **Ashburn, VA (US East)** |
| Server type | **CPX22** — 3 vCPU / 4 GB RAM / 80 GB disk (the ruled CPX22 class) |
| Operating system | **Ubuntu 24.04 LTS** (plain image) |
| Name | e.g. `rhl-ergon-host-01` |
| SSH login | Add **one** SSH key: paste the single line the team gives you (it is a **public** key, not a secret) |
| Networking | defaults; no extra services, no volumes, no backups add-on needed now |
| Cost | **about €8/month** (billed by Hetzner to your account) |

## Exactly what you do (you will see these screens)

1. Sign in to Hetzner Cloud, or create an account if you do not have one (account creation is the only extra step; it needs
   your email + payment method — your call, your account).
2. Click **New Server**.
3. Choose **Location: Ashburn, VA**; **Image: Ubuntu 24.04**; **Type: CPX22 (Shared vCPU)**.
4. Name it `rhl-ergon-host-01`.
5. In **SSH keys**, click **Add SSH key** and paste the public-key line the team sends you (a long line starting `ssh-ed25519`
   — it is safe to paste; it is not a password and gives nothing away).
6. Click **Create & Buy now**. Wait until the server shows **Running** with a public IP address.
7. Send the team the **server's public IP address** (visible at the top of the server page). That address is not a secret.

**Stop there.** Do not install anything, do not open the console, do not paste a password or a key anywhere else. The team
provisions and hardens the host from this point.

## Why you can trust the "paste this public key" step

A public SSH key is the lock, not the key: whoever holds the matching private key (the team, on a protected on-disk file it
never prints or copies) can log in; the public line alone lets nobody in. That is why this is the one credential-shaped
step you can do without sharing a secret. If you would rather not paste even that, the alternative is the same server and
the team sends you a one-line command to run in Hetzner's browser console — but that is more of your time, not less.

## What happens after you create it (team's side, so you know the next touchpoint)

The team installs the management software + DeepSeek Harness + the Ergon plugin on this host (once the package exists —
that build is a separate decision on your desk), wires the company database, hardens the public-facing side, and brings you
the shareable link and the first-employee enrolment steps. If anything will not be ready for the week of 2 Nov, you get the
named slippage list, not a surprise.

## Decision wording for the consolidated ask

> **Host purchase (recommended now if you want the package build to start):** create one Hetzner CPX22 server in Ashburn,
> Ubuntu 24.04, ~€8/month, and send the team its IP. Your act is only the purchase + pasting a public key. If you do not
> buy it yet, the team can rehearse on a throwaway local VM first, but the real deployment date cannot be set.
