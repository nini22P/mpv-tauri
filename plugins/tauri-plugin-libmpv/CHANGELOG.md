## v0.1.1

- Remove unused rendering modes.

## v0.1.0

- Replace the `libmpv2` dependency with a custom `libmpv-sys` binding.
- **BREAKING:** Overhauled the event type definitions to refine and update parameter structures. **Users must adapt event handlers to the new parameter layout.**
- Set the `LC_NUMERIC` on setup.
- Enhance type inference for `observeProperties` and `getProperty`
- Allow overriding `wid` option.
- Re-license to MPL-2.0.

## v0.0.1

- Frist release
