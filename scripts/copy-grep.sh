#!/usr/bin/env bash
# copy-grep — Gate B item B15.
# Scans BUILT HTML (not source) for claims we are not allowed to make.
#
# Written after a real finding: a naive token list flags the page's own honesty.
# "No APY on this page." / "What's the APY?" / "unaudited" / "April 2026" all
# contain banned substrings and are all correct copy. So the rules below are
# claim-shaped, with explicit allowances, and every allowance is justified here.
#
# usage: scripts/copy-grep.sh dist/ [more dirs...]
set -uo pipefail

DIRS=("$@"); [ ${#DIRS[@]} -eq 0 ] && DIRS=(dist)
FAIL=0
SEP='::'

# Non-greedy, multi-line aware. The original single-line sed was greedy: on
# server-rendered HTML that arrives as ONE line it deleted everything between
# the first <script> (theme boot in <head>) and the last </script> (module
# loader before </body>), leaving one word per page and a vacuous "clean".
# Found 2026-09-01 against apps/web's rendered routes; see docs/FACTS.md
# COPY_GREP_APP.
# Attribute text (meta descriptions, og:*, alt, aria-label, title) is visible to
# crawlers, link previews and screen readers, so it is pulled into the text
# before tags are removed.
strip_html() {
  # attr="value" becomes >value< : the value is moved outside its tag before
  # tags are removed, otherwise the tag stripper would eat it with the tag.
  perl -0pe 's/\b(?:content|alt|aria-label|title)\s*=\s*"([^"]*)"/>$1</gi; s/<script\b[^>]*>.*?<\/script>//gis; s/<style\b[^>]*>.*?<\/style>//gis; s/<!--.*?-->//gs; s/<[^>]*>/ /g' "$1"
}
# Words with only tags removed (script/style bodies kept). A page whose full
# strip keeps almost nothing of this is a broken strip; a page that is simply
# terse keeps most of it.
tagsonly_words() {
  perl -0pe 's/<[^>]*>/ /g' "$1" | tr -d '\000' | wc -w
}

# Self-test: a stripped page must keep its visible words. Fails loudly if a
# rendered page collapses to nothing after stripping, which is the failure mode
# above and would otherwise pass every rule.
MIN_WORDS=${COPY_GREP_MIN_WORDS:-40}

# name :: extended-regex (case-insensitive) :: why it is banned
RULES=(
  "promised-rate::[0-9]+(\.[0-9]+)?[[:space:]]*%[[:space:]]*(apy|apr)::a number next to APY or APR is a promised rate"
  "apy-claim::(earn|earning|returns?|yields?|pays?|paying|up to)[[:space:]]+[^.]{0,24}(apy|apr)::APY framed as something earned"
  "annualized::annuali[sz]ed::no annualisation on devnet marks"
  "guaranteed::guarantee(d|s)?::nothing here is guaranteed"
  "risk-free::risk[- ]free::no"
  "audited-claim::(is|been|fully|independently)[[:space:]]+audited::the program is unaudited; only 'unaudited' may appear"
  "mainnet::mainnet::Gate B is devnet only"
  "named-venue::(jupiter|drift|velocity|pacifica|hyperliquid)::no venue is named until the ADR-03 checklist passes"
  "eleven-refusals::all eleven::mechanism bragging, unverifiable in public copy"
)

# Allowed exactly as written; removed before the rules run.
ALLOW=(
  "No APY on this page."
  "What's the APY?"
  "There isn't one on this page."
  "unaudited"
  "Unaudited"
)

for dir in "${DIRS[@]}"; do
  while IFS= read -r -d '' f; do
    text=$(strip_html "$f" | tr -d '\000' | tr '\n' ' ' | tr -s ' ')
    words=$(printf '%s' "$text" | wc -w)
    raw=$(tagsonly_words "$f")
    if [ "$words" -lt "$MIN_WORDS" ] && [ "$words" -lt $(( raw / 10 )) ]; then
      printf 'FAIL  %-15s %s\n      rule: %s visible words after stripping vs %s with tags only removed; the strip is eating the page\n' "too-few-words" "$f" "$words" "$raw"
      FAIL=1
    fi
    for a in "${ALLOW[@]}"; do text=${text//"$a"/ }; done
    for rule in "${RULES[@]}"; do
      name=${rule%%${SEP}*}
      rest=${rule#*${SEP}}
      rx=${rest%%${SEP}*}
      why=${rest#*${SEP}}
      if printf '%s' "$text" | grep -Eqi -- "$rx"; then
        hit=$(printf '%s' "$text" | grep -Eoi -- ".{0,40}${rx}.{0,40}" | head -1)
        printf 'FAIL  %-15s %s\n      rule: %s\n      hit : %s\n' "$name" "$f" "$why" "$hit"
        FAIL=1
      fi
    done
  done < <(find "$dir" -name '*.html' -print0)
done

[ "$FAIL" -eq 0 ] && echo "copy-grep: clean"
exit "$FAIL"
