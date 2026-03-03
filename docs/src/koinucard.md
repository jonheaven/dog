# Koinucard

A **Koinucard** is a physical Dogecoin bearer card — load koinu onto it, hand
it to someone, done. No app, no wallet setup, no seed phrase.

The concept is inspired by Coinkite's
[SatCard](https://getsatscard.com) for Bitcoin. Coinkite profits by
manufacturing and selling physical SatCards; the sweep and verification
protocol they use is openly documented and anyone can implement it.
Nobody has done the equivalent for Dogecoin yet.

## How it works

1. A card is manufactured with an NFC chip containing a freshly generated
   private key.
2. The owner sends DOGE to the corresponding address — the card is now
   "loaded."
3. To spend, the recipient taps or scans the card, which reveals a URL
   encoding the card's public key and chain state.
4. `dog server` parses that URL at the `/koinucard` endpoint, shows the
   balance, and lets the holder sweep the funds to their own wallet.

The card is **sealed** (untouched) or **unsealed** (already swept). Anyone
can verify the state on-chain before accepting it.

## What's already built

The `dog` indexer ships full Koinucard support out of the box:

- URL parsing and address recovery — `src/koinucard.rs`
- Web UI for balance display and sweep — `src/templates/koinucard.rs`
- `dog server` hosts `/koinucard` — no extra setup needed

## What still needs to be built

The software is done. What remains is a hardware + distribution story:

- NFC card manufacturing (standard ISO 14443 chips work)
- Key generation and attestation at manufacture time
- Physical card design (Dogecoin branding, Shiba artwork — natural fit)
- A shop / distribution channel

## This is open — no single owner

Koinucard is a **protocol standard**, not a brand monopoly. Think of it
like gift cards: many different companies can print and sell gift cards, and
they all work in the same ecosystem. The same applies here.

- The `/koinucard` endpoint in `dog server` will sweep any card that follows
  the URL format — regardless of who manufactured it.
- Anyone can manufacture Koinucard-compatible NFC cards. Multiple competing
  vendors are the ideal outcome: more variety, better prices, no single
  point of control.
- The `dog` codebase defines the open standard. No registration, no
  licensing, no permission needed to make compatible cards.

Dogecoin is a community chain. The hardware side of Koinucards belongs to
whoever builds it — and that can be many people at once, just like there are
many Dogecoin wallets and exchanges without any one of them owning "Dogecoin."

The name **Koinucard** follows the project's Bitcoin → Dogecoin terminology
map (satoshi → koinu, SatCard → Koinucard) and is part of this open ecosystem.
