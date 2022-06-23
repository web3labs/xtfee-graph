use std::u128;

use pallet_transaction_payment::FeeDetails;
use subxt::sp_runtime::OpaqueExtrinsic;

use super::rpc_ext::NumberOrHex;
use plotters::prelude::*;

use primitive_types::H256;

#[derive(Clone, Debug)]
pub struct Block {
    pub timestamp: u128,
    pub block_hash: H256,
    pub extrinsics: Vec<Extrinsic>,
}

#[derive(Clone, Debug)]
pub struct Extrinsic {
    pub body: OpaqueExtrinsic,
    pub fee_details: FeeDetails<NumberOrHex>,
}

pub fn plot(blocks: Vec<Block>, output_path: String) -> Result<(), Box<dyn std::error::Error>> {
    let blocks_with_extrinsics: Vec<Block> = blocks
        .iter()
        .map(|x| Block {
            block_hash: x.block_hash,
            timestamp: x.timestamp,
            extrinsics: x
                .extrinsics
                .iter()
                .filter(|&intr| intr.fee_details.inclusion_fee.is_some())
                .cloned()
                .collect(),
        })
        .filter(|x| !x.extrinsics.is_empty())
        .collect();

    println!("Not empty blocks len: {}", blocks_with_extrinsics.len());

    let mut data = Vec::<u128>::new();
    blocks_with_extrinsics.iter().for_each(|block| {
        block.extrinsics.iter().for_each(|xtr| {
            match xtr
                .fee_details
                .inclusion_fee
                .as_ref()
                .unwrap()
                .adjusted_weight_fee
            {
                NumberOrHex::Number(number) => {
                    data.push(number as u128);
                }
                NumberOrHex::Hex(hex) => data.push(hex.as_u128()),
            }
        })
    });

    draw(data, output_path)
}

pub fn draw(data: Vec<u128>, output_path: String) -> Result<(), Box<dyn std::error::Error>> {
    let data_len = data.len();
    let data_max = *data.iter().max().unwrap_or(&0u128);
    let data_min = *data.iter().min().unwrap_or(&0u128);

    let root = BitMapBackend::new(&output_path, (960, 640)).into_drawing_area();
    root.fill(&WHITE)?;
    let mut chart = ChartBuilder::on(&root)
        .caption("Fee Observations", ("sans-serif", 50).into_font())
        .margin(5)
        .x_label_area_size(50)
        .y_label_area_size(120)
        .build_cartesian_2d(0..data_len, data_min..data_max)?;

    chart
        .configure_mesh()
        .x_desc("Transactions")
        .y_desc("Fees")
        .axis_desc_style(("sans-serif", 30).into_font())
        .draw()?;

    chart
        .draw_series(LineSeries::new(
            data.iter().enumerate().map(|(i, x)| (i, *x)),
            &RED,
        ))?
        .label("Fee Cost")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &RED));

    chart
        .configure_series_labels()
        .background_style(&WHITE.mix(0.8))
        .border_style(&BLACK)
        .draw()?;

    root.present()?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::plotter::draw;
    use rand::Rng;

    #[test]
    fn draw_nothing() {
        draw(vec![], "plotters-doc-data/draw_nothing.png".into()).unwrap();
    }

    #[test]
    fn draw_one_point() {
        draw(
            vec![318175599614067u128, 318175599614067u128],
            "plotters-doc-data/draw_one_point.png".into(),
        )
        .unwrap();
    }

    #[test]
    fn draw_linear() {
        draw(
            (0..10000).map(|v| v + 10000000000000).collect(),
            "plotters-doc-data/draw_linear.png".into(),
        )
        .unwrap();
    }

    #[test]
    fn draw_random() {
        let mut rng = rand::thread_rng();

        draw(
            (0..10000)
                .map(|_| rng.gen_range(10000000000000, 1000000000000000))
                .collect(),
            "plotters-doc-data/draw_random.png".into(),
        )
        .unwrap();
    }
}
