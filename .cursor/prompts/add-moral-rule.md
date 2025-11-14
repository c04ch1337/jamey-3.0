# Task: Add New Moral Rule
When adding a new moral rule to the conscience engine:
1. Add to `load_defaults()` function in conscience/mod.rs
2. Include: id (snake_case), weight (f32), description
3. Update any relevant tests
4. Consider how it interacts with existing rules
5. Document the rule's purpose and weighting rationale
