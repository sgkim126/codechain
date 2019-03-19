// Copyright 2019 Kodebox, Inc.
// This file is part of CodeChain.
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as
// published by the Free Software Foundation, either version 3 of the
// License, or (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use std::collections::BTreeMap;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use parking_lot::Mutex;
use time;

pub type MetricKey = &'static str;

#[derive(Default)]
pub struct MetricEntry {
    times: Vec<Duration>,
}

impl MetricEntry {
    fn add(&mut self, time: Duration) {
        self.times.push(time);
    }

    fn reset_prev(&mut self) {
        self.times.clear();
    }
}

#[derive(Default)]
pub struct Metric {
    inner: Arc<Mutex<MetricInner>>,
}

#[derive(Default)]
pub struct MetricInner {
    table: BTreeMap<MetricKey, MetricEntry>,
}

impl Metric {
    pub fn start_thread(&self) {
        let inner = Arc::clone(&self.inner);
        thread::Builder::new()
            .name("Metric logger".to_string())
            .spawn(move || loop {
                thread::sleep(::std::time::Duration::new(1, 0));
                inner.lock().print();
            })
            .unwrap();
    }

    pub fn add(&self, key: &'static str, time: Duration) {
        let mut guard = self.inner.lock();
        guard.add(key, time);
    }

    pub fn print(&self) {
        let mut guard = self.inner.lock();
        guard.print();
    }
}

impl MetricInner {
    pub fn add(&mut self, key: &'static str, time: Duration) {
        self.table.entry(key).or_default().add(time);
    }

    pub fn print(&mut self) {
        let timestamp = time::strftime("%Y-%m-%d %H:%M:%S %Z", &time::now()).unwrap();
        println!("Metric at : {}", timestamp);
        for (k, v) in self.table.iter_mut().filter(|(_, v)| !v.times.is_empty()) {
            {
                let total: Duration = v.times.iter().sum::<Duration>();
                let average = total / v.times.len() as u32;
                let max = v.times.iter().max().unwrap();
                let min = v.times.iter().min().unwrap();
                println!("{}: total: {:?} average: {:?} max: {:?} min: {:?} count: {}", k, total, average, max, min, v.times.len());
            }
            v.reset_prev();
        }
    }
}
