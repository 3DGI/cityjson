Overflow in a high-volume streaming test with 100_000 buildings and `ResourceId32`.

## Problem

- ResourceId32 uses u16 for generations (max 65,535)
- The test removes CityObjects after processing to maintain stable memory
- With 100,000 buildings and aggressive reuse, the same slot gets reused 100,000 times
- After 65,535 reuses, the u16 generation counter would overflow

## Solution

Modified `DefaultResourcePool::add()` to prevent generation overflow:
- When a slot's generation reaches `u16::MAX`, the slot is retired
- Retired slots are not reused, preventing wraparound to generation 0
- New slots are allocated when all free slots are retired

## Memory Implications

**Before fix:**
- Memory: O(max_concurrent_resources)
- Test case: 1 slot reused 100,000 times

**After fix:**
- Memory: O(max_concurrent_resources + retired_slots)
- Test case: ~34,465 total slots (1 slot × 65,536 reuses + 34,464 new slots)
- General: For N operations with single-slot reuse: ⌊N / 65,536⌋ retired slots
- Retired slots remain allocated for the pool's lifetime
- 1M buildings:
  - Retired slots: 1M / 65,536 = ~15 slots
  - Memory per retired slot: sizeof(Option<T>) discriminant ≈ 1-8 bytes
  - Total overhead: ~120 bytes
- 100M buildings:
  - Retired slots: ~1,526 slots
  - Memory: ~12 KB


**Trade-offs:**
- Prevents generation overflow and invalid references
- No impact for typical use cases (< 65K reuses per slot)
- Memory grows with extreme reuse patterns
- Retired slots are never reclaimed (bounded memory leak)
