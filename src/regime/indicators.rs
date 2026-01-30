//! Technical Indicators for Regime Detection
//!
//! Implements ADX, Bollinger Bands, ATR, and EMA calculations

use std::collections::VecDeque;

/// Exponential Moving Average calculator
#[derive(Debug, Clone)]
pub struct EMA {
    period: usize,
    multiplier: f64,
    current_value: Option<f64>,
    initialized: bool,
    warmup_count: usize,
}

impl EMA {
    pub fn new(period: usize) -> Self {
        let multiplier = 2.0 / (period as f64 + 1.0);
        Self {
            period,
            multiplier,
            current_value: None,
            initialized: false,
            warmup_count: 0,
        }
    }

    pub fn update(&mut self, price: f64) -> Option<f64> {
        self.warmup_count += 1;

        match self.current_value {
            Some(prev_ema) => {
                let new_ema = (price - prev_ema) * self.multiplier + prev_ema;
                self.current_value = Some(new_ema);

                if self.warmup_count >= self.period {
                    self.initialized = true;
                }
            }
            None => {
                self.current_value = Some(price);
            }
        }

        if self.initialized {
            self.current_value
        } else {
            None
        }
    }

    pub fn value(&self) -> Option<f64> {
        if self.initialized {
            self.current_value
        } else {
            None
        }
    }

    pub fn is_ready(&self) -> bool {
        self.initialized
    }
}

/// Average True Range (ATR) calculator
#[derive(Debug, Clone)]
pub struct ATR {
    period: usize,
    values: VecDeque<f64>,
    prev_close: Option<f64>,
    current_atr: Option<f64>,
}

impl ATR {
    pub fn new(period: usize) -> Self {
        Self {
            period,
            values: VecDeque::with_capacity(period),
            prev_close: None,
            current_atr: None,
        }
    }

    /// Update with OHLC data
    pub fn update(&mut self, high: f64, low: f64, close: f64) -> Option<f64> {
        let true_range = match self.prev_close {
            Some(prev_c) => {
                let hl = high - low;
                let hc = (high - prev_c).abs();
                let lc = (low - prev_c).abs();
                hl.max(hc).max(lc)
            }
            None => high - low,
        };

        self.prev_close = Some(close);
        self.values.push_back(true_range);

        if self.values.len() > self.period {
            self.values.pop_front();
        }

        if self.values.len() >= self.period {
            // Use Wilder's smoothing method
            match self.current_atr {
                Some(prev_atr) => {
                    let new_atr =
                        (prev_atr * (self.period - 1) as f64 + true_range) / self.period as f64;
                    self.current_atr = Some(new_atr);
                }
                None => {
                    let sum: f64 = self.values.iter().sum();
                    self.current_atr = Some(sum / self.period as f64);
                }
            }
        }

        self.current_atr
    }

    pub fn value(&self) -> Option<f64> {
        self.current_atr
    }

    pub fn is_ready(&self) -> bool {
        self.current_atr.is_some()
    }
}

/// Average Directional Index (ADX) calculator
/// Measures trend strength (not direction)
#[derive(Debug, Clone)]
pub struct ADX {
    period: usize,
    atr: ATR,
    plus_dm_ema: EMA,
    minus_dm_ema: EMA,
    dx_values: VecDeque<f64>,
    prev_high: Option<f64>,
    prev_low: Option<f64>,
    current_adx: Option<f64>,
    plus_di: Option<f64>,
    minus_di: Option<f64>,
}

impl ADX {
    pub fn new(period: usize) -> Self {
        Self {
            period,
            atr: ATR::new(period),
            plus_dm_ema: EMA::new(period),
            minus_dm_ema: EMA::new(period),
            dx_values: VecDeque::with_capacity(period),
            prev_high: None,
            prev_low: None,
            current_adx: None,
            plus_di: None,
            minus_di: None,
        }
    }

    pub fn update(&mut self, high: f64, low: f64, close: f64) -> Option<f64> {
        // Calculate directional movement
        let (plus_dm, minus_dm) = match (self.prev_high, self.prev_low) {
            (Some(prev_h), Some(prev_l)) => {
                let up_move = high - prev_h;
                let down_move = prev_l - low;

                let plus = if up_move > down_move && up_move > 0.0 {
                    up_move
                } else {
                    0.0
                };

                let minus = if down_move > up_move && down_move > 0.0 {
                    down_move
                } else {
                    0.0
                };

                (plus, minus)
            }
            _ => (0.0, 0.0),
        };

        self.prev_high = Some(high);
        self.prev_low = Some(low);

        // Update ATR
        let atr = self.atr.update(high, low, close);

        // Smooth directional movement
        let smoothed_plus_dm = self.plus_dm_ema.update(plus_dm);
        let smoothed_minus_dm = self.minus_dm_ema.update(minus_dm);

        // Calculate DI values
        if let (Some(atr_val), Some(plus_dm_smooth), Some(minus_dm_smooth)) =
            (atr, smoothed_plus_dm, smoothed_minus_dm)
        {
            if atr_val > 0.0 {
                self.plus_di = Some((plus_dm_smooth / atr_val) * 100.0);
                self.minus_di = Some((minus_dm_smooth / atr_val) * 100.0);

                // Calculate DX
                let di_sum = self.plus_di.unwrap() + self.minus_di.unwrap();
                if di_sum > 0.0 {
                    let di_diff = (self.plus_di.unwrap() - self.minus_di.unwrap()).abs();
                    let dx = (di_diff / di_sum) * 100.0;

                    self.dx_values.push_back(dx);
                    if self.dx_values.len() > self.period {
                        self.dx_values.pop_front();
                    }

                    // Calculate ADX as smoothed DX
                    if self.dx_values.len() >= self.period {
                        match self.current_adx {
                            Some(prev_adx) => {
                                let new_adx =
                                    (prev_adx * (self.period - 1) as f64 + dx) / self.period as f64;
                                self.current_adx = Some(new_adx);
                            }
                            None => {
                                let sum: f64 = self.dx_values.iter().sum();
                                self.current_adx = Some(sum / self.period as f64);
                            }
                        }
                    }
                }
            }
        }

        self.current_adx
    }

    pub fn value(&self) -> Option<f64> {
        self.current_adx
    }

    pub fn plus_di(&self) -> Option<f64> {
        self.plus_di
    }

    pub fn minus_di(&self) -> Option<f64> {
        self.minus_di
    }

    /// Returns trend direction based on DI crossover
    pub fn trend_direction(&self) -> Option<super::TrendDirection> {
        match (self.plus_di, self.minus_di) {
            (Some(plus), Some(minus)) => {
                if plus > minus {
                    Some(super::TrendDirection::Bullish)
                } else {
                    Some(super::TrendDirection::Bearish)
                }
            }
            _ => None,
        }
    }

    pub fn is_ready(&self) -> bool {
        self.current_adx.is_some()
    }
}

/// Bollinger Bands calculator
#[derive(Debug, Clone)]
pub struct BollingerBands {
    period: usize,
    std_dev_multiplier: f64,
    prices: VecDeque<f64>,
    width_history: VecDeque<f64>,
    width_history_size: usize,
}

impl BollingerBands {
    pub fn new(period: usize, std_dev_multiplier: f64) -> Self {
        Self {
            period,
            std_dev_multiplier,
            prices: VecDeque::with_capacity(period),
            width_history: VecDeque::with_capacity(100),
            width_history_size: 100, // Keep 100 periods for percentile calc
        }
    }

    pub fn update(&mut self, price: f64) -> Option<BollingerBandsValues> {
        self.prices.push_back(price);
        if self.prices.len() > self.period {
            self.prices.pop_front();
        }

        if self.prices.len() < self.period {
            return None;
        }

        // Calculate SMA (middle band)
        let sum: f64 = self.prices.iter().sum();
        let sma = sum / self.period as f64;

        // Calculate standard deviation
        let variance: f64 =
            self.prices.iter().map(|p| (p - sma).powi(2)).sum::<f64>() / self.period as f64;
        let std_dev = variance.sqrt();

        // Calculate bands
        let upper = sma + (std_dev * self.std_dev_multiplier);
        let lower = sma - (std_dev * self.std_dev_multiplier);
        let width = (upper - lower) / sma * 100.0; // Width as percentage of price

        // Update width history for percentile calculation
        self.width_history.push_back(width);
        if self.width_history.len() > self.width_history_size {
            self.width_history.pop_front();
        }

        // Calculate width percentile
        let width_percentile = self.calculate_width_percentile(width);

        // Calculate %B (where price is within bands)
        let percent_b = if upper - lower > 0.0 {
            (price - lower) / (upper - lower)
        } else {
            0.5
        };

        Some(BollingerBandsValues {
            upper,
            middle: sma,
            lower,
            width,
            width_percentile,
            percent_b,
            std_dev,
        })
    }

    fn calculate_width_percentile(&self, current_width: f64) -> f64 {
        if self.width_history.len() < 10 {
            return 50.0; // Not enough data
        }

        let count_below = self
            .width_history
            .iter()
            .filter(|&&w| w < current_width)
            .count();

        (count_below as f64 / self.width_history.len() as f64) * 100.0
    }

    pub fn is_ready(&self) -> bool {
        self.prices.len() >= self.period
    }
}

/// Bollinger Bands output values
#[derive(Debug, Clone, Copy)]
pub struct BollingerBandsValues {
    pub upper: f64,
    pub middle: f64,
    pub lower: f64,
    pub width: f64,            // Band width as % of price
    pub width_percentile: f64, // Where current width ranks historically
    pub percent_b: f64,        // Where price is within bands (0-1)
    pub std_dev: f64,
}

impl BollingerBandsValues {
    /// Is price overbought (near or above upper band)?
    pub fn is_overbought(&self) -> bool {
        self.percent_b >= 0.95
    }

    /// Is price oversold (near or below lower band)?
    pub fn is_oversold(&self) -> bool {
        self.percent_b <= 0.05
    }

    /// Are bands wide (high volatility)?
    pub fn is_high_volatility(&self, threshold_percentile: f64) -> bool {
        self.width_percentile >= threshold_percentile
    }

    /// Are bands narrow (potential breakout coming)?
    pub fn is_squeeze(&self, threshold_percentile: f64) -> bool {
        self.width_percentile <= threshold_percentile
    }
}

/// Simple Moving Average helper
pub fn calculate_sma(prices: &[f64]) -> f64 {
    if prices.is_empty() {
        return 0.0;
    }
    prices.iter().sum::<f64>() / prices.len() as f64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ema_calculation() {
        let mut ema = EMA::new(10);

        // Warm up
        for i in 1..=10 {
            ema.update(i as f64 * 10.0);
        }

        assert!(ema.is_ready());
        let value = ema.value().unwrap();
        assert!(value > 50.0 && value < 100.0);
    }

    #[test]
    fn test_bollinger_bands() {
        let mut bb = BollingerBands::new(20, 2.0);

        // Feed price data
        for i in 1..=25 {
            let price = 100.0 + (i as f64 % 5.0);
            bb.update(price);
        }

        assert!(bb.is_ready());
    }

    #[test]
    fn test_adx_trending_detection() {
        let mut adx = ADX::new(14);

        // Simulate trending market (prices going up steadily)
        for i in 1..=50 {
            let high = 100.0 + i as f64 * 2.0;
            let low = 100.0 + i as f64 * 2.0 - 1.0;
            let close = 100.0 + i as f64 * 2.0 - 0.5;
            adx.update(high, low, close);
        }

        if let Some(adx_value) = adx.value() {
            println!("ADX value in uptrend: {}", adx_value);
            assert!(adx_value > 20.0, "ADX should indicate trend");
        }
    }
}
