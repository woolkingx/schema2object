"""Minimal grapheme cluster counter (pure Python).

Heuristic implementation that groups:
- combining marks
- variation selectors
- ZWJ sequences (emoji)

Good enough to align with JS/Rust for common emoji clusters.
"""
from __future__ import annotations

import unicodedata

ZWJ = "\u200d"
VARIATION_SELECTORS = {"\ufe0e", "\ufe0f"}


def _is_combining(ch: str) -> bool:
    return unicodedata.combining(ch) != 0 or unicodedata.category(ch) in {"Mn", "Mc"}


def grapheme_len(s: str) -> int:
    if not s:
        return 0
    count = 0
    i = 0
    n = len(s)
    while i < n:
        count += 1
        i += 1
        # consume combining marks and variation selectors
        while i < n and (_is_combining(s[i]) or s[i] in VARIATION_SELECTORS):
            i += 1
        # consume ZWJ sequences: ... + ZWJ + next + optional combining/VS
        while i < n - 1 and s[i] == ZWJ:
            i += 1  # skip ZWJ
            i += 1  # consume next base
            while i < n and (_is_combining(s[i]) or s[i] in VARIATION_SELECTORS):
                i += 1
    return count
