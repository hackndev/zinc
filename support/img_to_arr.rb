#!/usr/bin/env ruby

require 'chunky_png'
image = ChunkyPNG::Image.from_file '/Users/farcaller/Desktop/header-firefox.png'

open('img.rs', 'w') do |f|
  f.write(<<-EOF)

pub static image_width: u32 = #{image.width};
pub static image_height: u32 = #{image.height};
pub static image_data: &'static [u16] = &[
EOF

  image.height.times do |y|
    image.width.times do |x|
      pixel = image[x, y]

      # split
      r = (pixel & 0xff000000) >> 24
      g = (pixel & 0xff0000) >> 16
      b = (pixel & 0xff00) >> 8
      a = (pixel & 0xff)

      # scale to 5:6:5
      r5 = (0b11111.to_f * r / 255).floor
      g6 = (0b111111.to_f * g / 255).floor
      b5 = (0b11111.to_f * b / 255).floor

      out_pixel = (r5 << 11) | (g6 << 5) | b5

      f.write("#{out_pixel}, ")
    end
    f.write("\n")
  end

  f.write("];")
end
