/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under both the MIT license found in the
 * LICENSE-MIT file in the root directory of this source tree and the Apache
 * License, Version 2.0 found in the LICENSE-APACHE file in the root directory
 * of this source tree.
 */

//! Example that demonstrates stylization.

use std::time::Duration;

use derive_more::Display;
use superconsole::components::Component;
use superconsole::components::DrawMode;
use superconsole::style::style;
use superconsole::style::Color;
use superconsole::style::Stylize;
use superconsole::Dimensions;
use superconsole::Line;
use superconsole::Span;
use superconsole::SuperConsole;
use tokio::time;

/// A component representing a store greeter.
#[derive(Debug)]
struct Greeter {
    name: String,
}

#[derive(Clone, Debug, Display)]
struct StoreName(String);

#[derive(Clone, Debug, Display)]
struct CustomerName(String);

#[derive(Clone, Debug)]
struct GreeterState {
    store_name: StoreName,
    customers: Vec<CustomerName>,
}

impl Component<Option<GreeterState>> for Greeter {
    /// Prints a greeting to the current customer.
    fn draw_unchecked(
        &self,
        maybe_state: &Option<GreeterState>,
        _dimensions: Dimensions,
        mode: DrawMode,
    ) -> anyhow::Result<Vec<Line>> {
        Ok(match mode {
            DrawMode::Final => vec![],
            DrawMode::Normal => {
                if let Some(state) = maybe_state {
                    let identification = Line(vec![
                        "Hello my name is ".to_owned().italic().try_into()?,
                        style(self.name.clone()).bold().try_into()?,
                    ]);
                    let mut messages = vec![identification];
                    for customer_name in &state.customers {
                        let greeting = Line(vec![Span::new_styled(style(format!(
                            "Welcome to {}, {}!",
                            state.store_name, customer_name
                        )))?]);
                        messages.push(greeting);
                    }
                    messages
                } else {
                    vec![]
                }
            }
        })
    }
}

#[tokio::main]
async fn main() {
    let mut console = SuperConsole::new(Box::new(Greeter {
        name: "Alex".to_owned(),
    }))
    .unwrap();

    let people = [
        "Joseph", "Janet", "Bob", "Christie", "Raj", "Sasha", "Rayna", "Veronika", "Russel",
        "David",
    ];
    let store_names = [
        "Target",
        "Target",
        "Target",
        "TJ",
        "TJ",
        "Walmart",
        "Wendys",
        "Wendys",
        "Uwajimaya",
        "DSW",
    ];

    let mut timer = time::interval(Duration::from_secs_f32(0.5));
    for i in 0usize..10usize {
        let styled = i.to_string().with(Color::Green).on(Color::Black);
        console.emit(vec![Line(vec![styled.try_into().unwrap()])]);
        let customers = (i..std::cmp::min(10, i + 2))
            .map(|x| CustomerName(people[x].to_owned()))
            .collect::<Vec<_>>();
        let store_name = StoreName(store_names[i].to_owned());

        console
            .render(&Some(GreeterState {
                store_name: store_name.clone(),
                customers: customers.clone(),
            }))
            .unwrap();

        timer.tick().await;
    }

    // view the output before it's collapsed
    tokio::time::sleep(Duration::from_secs(1)).await;
    console.finalize(&None).unwrap();
}
