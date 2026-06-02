# Intern Benchmark Report

2 intern(s) graded across 6 tasks by the main model (LLM-as-judge), 0–10 scale, pass ≥ 6.

## Leaderboard

| Rank | Intern | Avg /10 | Total | Passed | Time |
|------|--------|---------|-------|--------|------|
| 1 | deepseek | 10.0 | 60/60 | 6/6 | 74.0s |
| 2 | mimo | 9.7 | 58/60 | 6/6 | 88.4s |

## Per-task scores

| Task | Category | deepseek | mimo |
|------|----------|------|------|
| palindrome | coding | 10 | 10 |
| binary-search | algorithm | 10 | 9 |
| find-bug | debugging | 10 | 9 |
| git-undo | instruction-following | 10 | 10 |
| logic | reasoning | 10 | 10 |
| refactor | refactor | 10 | 10 |

## Judge notes

### deepseek

- **palindrome** (coding) — 10/10, 13.3s — Correct logic, filters non-alphanumeric characters, ignores ASCII case, compiles as-is, and is idiomatic Rust with no extraneous code.
- **binary-search** (algorithm) — 10/10, 33.5s — Correct iterative binary search with O(log n) complexity, overflow-safe midpoint using lo + (hi - lo) / 2, handles empty/single/not-found/duplicate cases, and compiles cleanly.
- **find-bug** (debugging) — 10/10, 10.5s — Correctly identifies the zero-initialization bug causing wrong results for all-negative slices, provides two valid fixes (Option-returning with first-element initialization, and i32::MIN initialization), and notes the empty-slice edge case.
- **git-undo** (instruction-following) — 10/10, 2.7s — The answer is exactly `git reset --soft HEAD~1`, which is the correct command to undo the most recent commit while keeping changes staged, with no extra prose or commands.
- **logic** (reasoning) — 10/10, 7.1s — Correct answer of $0.05 with valid reasoning: bat = $1.05, ball = $0.05, difference is $1.00 and sum is $1.10.
- **refactor** (refactor) — 10/10, 6.9s — Perfect idiomatic iterator chain using filter and count, correct double-dereference pattern `&&x`, behavior preserved, compiles correctly, no imperative scaffolding remaining.

### mimo

- **palindrome** (coding) — 10/10, 46.2s — The function correctly filters non-alphanumeric characters, lowercases ASCII, and compares symmetric positions; it compiles as-is, handles all edge cases, and is idiomatic Rust with no extraneous code.
- **binary-search** (algorithm) — 9/10, 18.1s — The implementation is correct, compiles, uses half-open interval [low, high), avoids overflow with low+(high-low)/2, handles empty/one-element/not-found cases, and is O(log n). Minor deduction only because no actual file was written (zero files modified per ground truth), but the function itself is algorithmically sound and complete.
- **find-bug** (debugging) — 9/10, 7.9s — Correctly identifies the zero-initialization bug causing failure on all-negative slices and empty slices, and provides a correct fix initializing from the first element; minor deduction because the fix panics on empty slices without explicit handling, though this is acknowledged in the explanation.
- **git-undo** (instruction-following) — 10/10, 3.8s — The answer is exactly `git reset --soft HEAD~1`, which is the correct command to undo the most recent commit while keeping changes staged, with no extra prose or commands.
- **logic** (reasoning) — 10/10, 5.3s — Correct answer of $0.05 with valid verification showing ball + bat = $0.05 + $1.05 = $1.10 and bat exceeds ball by exactly $1.00.
- **refactor** (refactor) — 10/10, 7.1s — The solution uses an idiomatic iterator chain with filter and count, correctly handles the double-reference pattern with `&&x`, preserves behavior, and compiles correctly.

