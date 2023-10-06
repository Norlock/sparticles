#![allow(dead_code)]
use egui_wgpu::wgpu;

pub struct Performance {
    set: wgpu::QuerySet,
    resolve_buffer: wgpu::Buffer,
    destination_buffer: wgpu::Buffer,
    num_queries: u64,
    next_unused_query: u32,
}

impl Performance {
    fn new(device: &wgpu::Device, num_queries: u64) -> Self {
        Self {
            set: device.create_query_set(&wgpu::QuerySetDescriptor {
                label: Some("Timestamp query set"),
                count: num_queries as _,
                ty: wgpu::QueryType::Timestamp,
            }),
            resolve_buffer: device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("query resolve buffer"),
                size: std::mem::size_of::<u64>() as u64 * num_queries,
                usage: wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::QUERY_RESOLVE,
                mapped_at_creation: false,
            }),
            destination_buffer: device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("query dest buffer"),
                size: std::mem::size_of::<u64>() as u64 * num_queries,
                usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
                mapped_at_creation: false,
            }),
            num_queries,
            next_unused_query: 0,
        }
    }
}
