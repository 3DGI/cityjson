Overflow in the `test_producer_consumer_stream` with 100_000 buildings and `ResourceId32`.

- ResourceId32 uses u16 for generations (max 65,535)
- The test removes CityObjects after processing to maintain stable memory
- With 100,000 buildings and aggressive reuse, the same slot gets reused 100,000 times
- After 65,000 reuses, the u16 generation counter overflows