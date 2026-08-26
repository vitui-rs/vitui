---
status: accepted
date: 2026-08-26
---

# A collapsible's open height is an argument, never a measurement

A section's open height comes from a **sizing function beside the body** — `FnMut(u16) -> u16`, which
is the shape `vitui_runtime::sizing::check` already sweeps a component against. The alternative —
hand the body a rectangle and read how far it reached — is off by default and kept only as a priced
negative case (`disclose::Height::Watermark`).

And the machine has **two states and no third**. `Collapse::set` flips `open` at the instant the
gesture lands and starts a tween on the height, so `open` is never ambiguous and only the height
moves.

## Why

**A measured extent is taken inside the rectangle the decision produced.** This is
[ADR 0029](0029-scrollbars-are-reserved-not-overlaid.md)'s second sentence on the other axis, and the
generalisation is not a rhyme: it is the same feedback loop with a height in it instead of a width.
Fed back into itself over a body that fills what it is handed — which is what a padding ring *is* —
the height **latches at the first rectangle it ever saw and never comes down**: 19 rows where 4 are
right, permanently, on a screen where every row is drawn, the chevron is correct, and no counter
moves.

**The precondition is on the body, exactly as ADR 0029 states it.** Over a body that draws only its
content the two spellings are **indistinguishable** — same height, same frame, same cells. *Fills its
rectangle* is not a property any component can guarantee of its body, which is what makes the rule
unconditional rather than a default somebody may flip.

**Spec §8's own figures for the watermark are a prototype's body and do not reproduce.** It prices the
drawn extent at *83 cells, 8 rows wrong, settled in 3 frames* against a sizing function's *22, 0, 2*.
Neither reproduces here and neither is engineered to: what reproduces is the latch, which is stronger,
and the direction §8 was pointing at.

**The price is a count and not a clock.** §8 puts the watermark at *+11.5% of the frame*; measured in
this crate's own currency it is one extra pass over the body — `Ctx::measured` runs the body through
the caller's ink, so the verb count carries the probe. A timing is a report (§21, R15); a doubled verb
count is a gate.

**A transition state would have to be stored.** `Collapsing` and `Expanding` are the shape R11 refused
when it refused an animation object: a stored third state means the machine can be found halfway
between two states with no clock running. `set` writing `open` immediately makes the question
unanswerable rather than answered — and the absence is kept by a **source scan**, because *the item
does not exist* has no expression and a `compile_fail` naming a variant nobody built passes today and
passes again the day somebody adds one under a different name.

**An animated fold is refused, and the reason is the index rather than taste.** The removed rows would
have to still be in the index while they shrink, and the splice is an edit a frame may not perform —
`Vec::splice` allocates against a frame budget of zero, and taking `&mut` while the draw holds the
index shared is `E0502`. **A fold steps; a region animates**, and `Collapses::OnRequest` starts no
tween at all.

## Consequences

**Accordion, tree node, code folding and inplace edit are one machine and three configurations.** The
split is not between the four components; it is between what their collapsed content *is* — rows in a
caller-owned index, a region of components, or both — and that is `disclose::SPLIT` as a value with
`disclose::Collapses` as the join. *A component may not perform an edit; a collapse of a region is not
one.*

**A `Collapse` costs four bytes of live state and forty-eight with the tween slot**, and §8's `5 / 72`
is neither. The live half is `open: bool` and a `u16`; five is what a third `u16` would cost and there
is no third, because `Tween::to` is the target while a tween runs and the height is it afterwards.
**And the pair cannot both be a `size_of` of one type** — an `Option<Tween<u16>>` field costs its forty
bytes empty, so a record that is 5 B without a tween is a record whose tween lives somewhere else, and
nothing on this map has anywhere to put one.

**Closed content is not drawn, and the cells are not why.** A body called with an `h = 0` rectangle
instead of skipped declares **408 more hit entries and 408 more ring entries** on §8's own accordion —
one subtraction twice, because `Ctx::interact` appends to both before either looks at the rectangle —
on two surfaces that are 0 cells over 0 rows apart. Of spec §20's nine counters only `regions` and
`tab stops` see it.

**The rows a section vacates belong to whoever owns the rectangle.** `Disclosure::used` names them and
a caller that writes its tail from the height it had *before* the collapse keeps the old body on the
screen under a correct header: 240 cells over 4 rows, nothing thrown, screen plausible. This is spec
§9's seam sentence on the height axis.

**§8's *0 ring probes against 405* is not reachable from a header gesture.** All three self-close
gestures leave the focus off the body before R08's vanish rule looks — a click on a focusable header is
awarded the focus, a click on one that is not a tab stop **defocuses**, and `Enter` needs the header to
hold the focus already. The arm that pays belongs to a collapse nobody clicked for, and there the
answer is §8's other sentence: **`Drop` is the component's answer; `Stash` belongs to whoever owns the
content's identity**, and the caller does it by capturing `Frame::focus`. What focusing the header
actually buys is the keyboard: on a header that is not a tab stop the click's answer is `None`.
