## v0.1.0

- Replace the `libmpv2` dependency with a custom `libmpv-sys` binding.
- **BREAKING CHANGE:** Overhauled the event type definitions to refine and update parameter structures. **Users must adapt event handlers to the new parameter layout.**
- Set the `LC_NUMERIC` locale on setup.
- Re-license to MPL-2.0.
- Enhance type inference for `observeProperties` and `getProperty`

## v0.0.1

- Frist release
