# A component-based framework for building Rust TUIs

The superconsole framework provides a powerful line based abstraction over text based rendering to the terminal.  It also provides basic building blocks like line manipulation, and a higher level of composable components.  A base set of "batteries" components are included to help developers create TUIs as quickly as possible.

The design choices that underly superconsole are selected to prioritize testability, ease of composition, and flexibility.

Superconsole also offers stylization, including italics, underlining, bolding, and coloring text.  Furthermore, relying on crossterm ensures that it is compatible with Windows, Unix, and MacOS.

Finally, superconsole delineates between rendering logic and program state - each render call acceps an immutable reference to state, which components may use to inject state into their otherwise immutable rendering logic.

## Demo

![Superconsole running some buck2 tests](demo.gif)


## Examples

```rust
struct HelloWorld;

impl Component for HelloWorld {
    fn draw_unchecked(
        &self,
        state: &State,
        dimensions: Dimensions,
        mode: DrawMode,
    ) -> anyhow::Result<Vec<Line>> {
        Ok(vec![vec!["hello world"].try_into()?])
    }
}

pub fn main() {
    let bordering = BorderedSpec::default();
    let mut superconsole = Superconsole::new(box Bordered::new(box HelloWorld, bordering)).unwrap();
    let _res = superconsole.render(&state![]).unwrap();
}
```

## Requirements

Superconsole works with MacOS, Linux, and Windows.
Can only be run in a tty.

See the [CONTRIBUTING](CONTRIBUTING.md) file for how to help out.

## License

Superconsole is both MIT and Apache License, Version 2.0 licensed, as found in the [LICENSE-MIT](LICENSE-MIT) and [LICENSE-APACHE](LICENSE-APACHE) files.
