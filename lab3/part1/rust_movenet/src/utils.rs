use opencv::{
	prelude::*,
	imgproc::*,
	core::*,
};

pub fn resize_with_padding(img: &Mat, new_shape: [i32;2]) -> Mat {
	let img_shape = [img.cols(), img.rows()];
	let width: i32;
	let height: i32;
	if img_shape[0] as f64 / img_shape[1] as f64 > new_shape[0] as f64 / new_shape[1] as f64 {
		width = new_shape[0];
		height = (new_shape[0] as f64 / img_shape[0] as f64 * img_shape[1] as f64) as i32;
	} else {
		width = (new_shape[1] as f64 / img_shape[1] as f64 * img_shape[0] as f64) as i32;
		height = new_shape[1];
	}

	let mut resized = Mat::default();
	resize(
		img,
		&mut resized,
		Size { width, height },
		0.0, 0.0,
		INTER_LINEAR)
		.expect("resize_with_padding: resize [FAILED]");

	let delta_w = new_shape[0] - width;
	let delta_h = new_shape[1] - height;
	let (top, bottom) = (delta_h / 2, delta_h - delta_h / 2);
	let (left, right) = (delta_w / 2, delta_w - delta_w / 2);
		
	let mut rslt = Mat::default();
	copy_make_border(
		&resized,
		&mut rslt,
		top, bottom, left, right,
		BORDER_CONSTANT,
		Scalar::new(0.0, 0.0, 0.0, 0.0))
		.expect("resize_with_padding: copy_make_border [FAILED]");
	rslt
}

pub fn draw_keypoints(img: &mut Mat, keypoints: &[f32], threshold: f32) {
	// keypoints: [1, 17, 3]
	let base: f32;
	let pad_x: i32;
	let pad_y: i32;
	if img.rows() > img.cols() {
		base = img.rows() as f32;
		pad_x = (img.rows() - img.cols()) / 2;
		pad_y = 0;
	} else {
		base = img.cols() as f32;
		pad_x = 0;
		pad_y = (img.cols() - img.rows()) / 2;
	}

	for index in 0..17 {
		let y_ratio = keypoints[index * 3];
		let x_ratio = keypoints[index * 3 + 1];
		let confidence = keypoints[index * 3 + 2];
		if confidence > threshold {
			circle(img,
				Point { x: (x_ratio * base) as i32 - pad_x, y: (y_ratio * base) as i32 - pad_y},
				0,
				Scalar::new(0.0, 255.0, 0.0, 0.0),
				5, LINE_AA, 0).expect("Draw circle [FAILED]");
		}
	}
}

fn YUV_2_B(y: i32, u: i32) -> i32 {
    let y = y as f32;
    let u = u as f32;
    (y + 1.732446 * (u - 128.0)) as i32
}
fn YUV_2_G(y: i32, u: i32, v: i32) -> i32 {
    let y = y as f32;
    let u = u as f32;
    let v = v as f32;
    (y - 0.698001 * (u - 128.0) - 0.703125 * (v - 128.0)) as i32
}
fn YUV_2_R(y: i32, v: i32) -> i32 {
    let y = y as f32;
    let v = v as f32;
    (y + 1.370705 * (v - 128.0)) as i32
}

// adapted from https://github.com/kd40629rtlrtl/yuv422_to_rgb/blob/master/yuv_to_rgb.c
pub fn yuv422_to_rgb24(in_buf: &[u8], out_buf: &mut [u8], width: i32, height: i32) {
    let len: i32 = width * height;
    let yData: &[u8] = in_buf;
    let vData: &[u8] = in_buf; 
    let uData: &[u8] = in_buf;

    let mut bgr: [i32; 3] = [0; 3];
    let mut yIdx: usize;
    let mut uIdx: usize;
    let mut vIdx: usize;
    let mut idx: usize;
    for y in 0..height {
        for x in 0..width {
           	yIdx = 2*((y*width) + x) as usize;
            uIdx = (4*(((y*width) + x)>>1) + 1) as usize;
            vIdx = (4*(((y*width) + x)>>1) + 3) as usize;

            if yIdx >= in_buf.len() || uIdx >= in_buf.len() || vIdx >= in_buf.len() {
                return; 
            }
            bgr[0] = YUV_2_B(yData[yIdx].into(), uData[uIdx].into()); 
			bgr[1] = YUV_2_G(yData[yIdx].into(), uData[uIdx].into(), vData[vIdx].into());
			bgr[2] = YUV_2_R(yData[yIdx].into(), vData[vIdx].into()); 

			for k in 0..3 as usize {
                idx = ((y * width + x) * 3 + (k as i32)) as usize;
                if bgr[k] >= 0 && bgr[k] <= 255 {
                    out_buf[idx] = bgr[k] as u8;
                } else if bgr[k] < 0 {
                    out_buf[idx] = 0;
                } else {
                    out_buf[idx] = 255;
                }
            }
        }
    }
}

