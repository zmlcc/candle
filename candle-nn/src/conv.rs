//! Convolution Layers.
use candle::{Result, Tensor};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Conv1dConfig {
    pub padding: usize,
    pub stride: usize,
    pub dilation: usize,
    pub groups: usize,
}

impl Default for Conv1dConfig {
    fn default() -> Self {
        Self {
            padding: 0,
            stride: 1,
            dilation: 1,
            groups: 1,
        }
    }
}

#[derive(Debug)]
pub struct Conv1d {
    weight: Tensor,
    bias: Option<Tensor>,
    config: Conv1dConfig,
}

impl Conv1d {
    pub fn new(weight: Tensor, bias: Option<Tensor>, config: Conv1dConfig) -> Self {
        Self {
            weight,
            bias,
            config,
        }
    }

    pub fn config(&self) -> &Conv1dConfig {
        &self.config
    }
}

impl crate::Module for Conv1d {
    fn forward(&self, x: &Tensor) -> Result<Tensor> {
        let x = x.conv1d(
            &self.weight,
            self.config.padding,
            self.config.stride,
            self.config.dilation,
            self.config.groups,
        )?;
        match &self.bias {
            None => Ok(x),
            Some(bias) => {
                let b = bias.dims1()?;
                let bias = bias.reshape((1, b, 1))?;
                Ok(x.broadcast_add(&bias)?)
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Conv2dConfig {
    pub padding: usize,
    pub stride: usize,
    pub dilation: usize,
    pub groups: usize,
}

impl Default for Conv2dConfig {
    fn default() -> Self {
        Self {
            padding: 0,
            stride: 1,
            dilation: 1,
            groups: 1,
        }
    }
}

#[derive(Debug)]
pub struct Conv2d {
    weight: Tensor,
    bias: Option<Tensor>,
    config: Conv2dConfig,
}

impl Conv2d {
    pub fn new(weight: Tensor, bias: Option<Tensor>, config: Conv2dConfig) -> Self {
        Self {
            weight,
            bias,
            config,
        }
    }

    pub fn config(&self) -> &Conv2dConfig {
        &self.config
    }
}

impl crate::Module for Conv2d {
    fn forward(&self, x: &Tensor) -> Result<Tensor> {
        let x = x.conv2d(
            &self.weight,
            self.config.padding,
            self.config.stride,
            self.config.dilation,
            self.config.groups,
        )?;
        match &self.bias {
            None => Ok(x),
            Some(bias) => {
                let b = bias.dims1()?;
                let bias = bias.reshape((1, b, 1, 1))?;
                Ok(x.broadcast_add(&bias)?)
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ConvTranspose2dConfig {
    pub padding: usize,
    pub output_padding: usize,
    pub stride: usize,
    pub dilation: usize,
    // TODO: support groups.
}

impl Default for ConvTranspose2dConfig {
    fn default() -> Self {
        Self {
            padding: 0,
            output_padding: 0,
            stride: 1,
            dilation: 1,
        }
    }
}

#[derive(Debug)]
pub struct ConvTranspose2d {
    weight: Tensor,
    bias: Option<Tensor>,
    config: ConvTranspose2dConfig,
}

impl ConvTranspose2d {
    pub fn new(weight: Tensor, bias: Option<Tensor>, config: ConvTranspose2dConfig) -> Self {
        Self {
            weight,
            bias,
            config,
        }
    }

    pub fn config(&self) -> &ConvTranspose2dConfig {
        &self.config
    }
}

impl crate::Module for ConvTranspose2d {
    fn forward(&self, x: &Tensor) -> Result<Tensor> {
        let x = x.conv_transpose2d(
            &self.weight,
            self.config.padding,
            self.config.output_padding,
            self.config.stride,
            self.config.dilation,
        )?;
        match &self.bias {
            None => Ok(x),
            Some(bias) => {
                let b = bias.dims1()?;
                let bias = bias.reshape((1, b, 1, 1))?;
                Ok(x.broadcast_add(&bias)?)
            }
        }
    }
}

pub fn conv1d(
    in_channels: usize,
    out_channels: usize,
    kernel_size: usize,
    cfg: Conv1dConfig,
    vs: crate::VarBuilder,
) -> Result<Conv1d> {
    let init_ws = crate::init::DEFAULT_KAIMING_NORMAL;
    let ws = vs.get_with_hints(
        (out_channels, in_channels / cfg.groups, kernel_size),
        "weight",
        init_ws,
    )?;
    let bound = 1. / (in_channels as f64).sqrt();
    let init_bs = crate::Init::Uniform {
        lo: -bound,
        up: bound,
    };
    let bs = vs.get_with_hints(out_channels, "bias", init_bs)?;
    Ok(Conv1d::new(ws, Some(bs), cfg))
}

pub fn conv2d(
    in_channels: usize,
    out_channels: usize,
    kernel_size: usize,
    cfg: Conv2dConfig,
    vs: crate::VarBuilder,
) -> Result<Conv2d> {
    let init_ws = crate::init::DEFAULT_KAIMING_NORMAL;
    let ws = vs.get_with_hints(
        (
            out_channels,
            in_channels / cfg.groups,
            kernel_size,
            kernel_size,
        ),
        "weight",
        init_ws,
    )?;
    let bound = 1. / (in_channels as f64).sqrt();
    let init_bs = crate::Init::Uniform {
        lo: -bound,
        up: bound,
    };
    let bs = vs.get_with_hints(out_channels, "bias", init_bs)?;
    Ok(Conv2d::new(ws, Some(bs), cfg))
}

pub fn conv2d_no_bias(
    in_channels: usize,
    out_channels: usize,
    kernel_size: usize,
    cfg: Conv2dConfig,
    vs: crate::VarBuilder,
) -> Result<Conv2d> {
    let init_ws = crate::init::DEFAULT_KAIMING_NORMAL;
    let ws = vs.get_with_hints(
        (
            out_channels,
            in_channels / cfg.groups,
            kernel_size,
            kernel_size,
        ),
        "weight",
        init_ws,
    )?;
    Ok(Conv2d::new(ws, None, cfg))
}

pub fn conv_transpose2d(
    in_channels: usize,
    out_channels: usize,
    kernel_size: usize,
    cfg: ConvTranspose2dConfig,
    vs: crate::VarBuilder,
) -> Result<ConvTranspose2d> {
    let bound = 1. / (out_channels as f64).sqrt() / kernel_size as f64;
    let init = crate::Init::Uniform {
        lo: -bound,
        up: bound,
    };
    let ws = vs.get_with_hints(
        (in_channels, out_channels, kernel_size, kernel_size),
        "weight",
        init,
    )?;
    let bs = vs.get_with_hints(out_channels, "bias", init)?;
    Ok(ConvTranspose2d::new(ws, Some(bs), cfg))
}

pub fn conv_transpose2d_no_bias(
    in_channels: usize,
    out_channels: usize,
    kernel_size: usize,
    cfg: ConvTranspose2dConfig,
    vs: crate::VarBuilder,
) -> Result<ConvTranspose2d> {
    let bound = 1. / (out_channels as f64).sqrt() / kernel_size as f64;
    let init = crate::Init::Uniform {
        lo: -bound,
        up: bound,
    };
    let ws = vs.get_with_hints(
        (out_channels, in_channels, kernel_size, kernel_size),
        "weight",
        init,
    )?;
    Ok(ConvTranspose2d::new(ws, None, cfg))
}
