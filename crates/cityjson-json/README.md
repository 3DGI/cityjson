### Data

Dependencies:
- [just](https://just.systems/)

The test data setup is managed with *just* from the root directory.
You need to init the data directories and download all test files:

```shell
just download
```

The downloaded files are placed into `serde-cityjson/resources/data/downloaded`.

Finally, you can clean up with `just clean`.
