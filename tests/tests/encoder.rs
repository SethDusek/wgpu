use wgpu_test::{fail, gpu_test, FailureCase, GpuTestConfiguration, TestParameters};

#[gpu_test]
static DROP_ENCODER: GpuTestConfiguration = GpuTestConfiguration::new().run_sync(|ctx| {
    let encoder = ctx
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
    drop(encoder);
});

// This test crashes on DX12 with the exception:
//
// ID3D12CommandAllocator::Reset: The command allocator cannot be reset because a
// command list is currently being recorded with the allocator. [ EXECUTION ERROR
// #543: COMMAND_ALLOCATOR_CANNOT_RESET]
//
// For now, we mark the test as failing on DX12.
#[gpu_test]
static DROP_ENCODER_AFTER_ERROR: GpuTestConfiguration = GpuTestConfiguration::new()
    .parameters(TestParameters::default().expect_fail(FailureCase::backend(wgpu::Backends::DX12)))
    .run_sync(|ctx| {
        let mut encoder = ctx
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

        let target_tex = ctx.device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: wgpu::Extent3d {
                width: 100,
                height: 100,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::R8Unorm,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });
        let target_view = target_tex.create_view(&wgpu::TextureViewDescriptor::default());

        let mut renderpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("renderpass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                ops: wgpu::Operations::default(),
                resolve_target: None,
                view: &target_view,
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        // Set a bad viewport on renderpass, triggering an error.
        fail(&ctx.device, || {
            renderpass.set_viewport(0.0, 0.0, -1.0, -1.0, 0.0, 1.0);
            drop(renderpass);
        });

        // This is the actual interesting error condition. We've created
        // a CommandEncoder which errored out when processing a command.
        // The encoder is still open!
        drop(encoder);
    });
