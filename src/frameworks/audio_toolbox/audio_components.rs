use std::collections::HashMap;
use std::time::Instant;

use touchHLE_openal_soft_wrapper::al_types::ALuint;

use crate::abi::GuestFunction;
use crate::dyld::FunctionExports;
use crate::environment::Environment;
use crate::export_c_func;
use crate::frameworks::carbon_core::OSStatus;
use crate::frameworks::core_audio_types::{
    fourcc, kAudioFormatFlagIsAlignedHigh, kAudioFormatFlagIsFloat, kAudioFormatFlagIsPacked,
    kAudioFormatFlagIsSignedInteger, kAudioFormatLinearPCM, AudioStreamBasicDescription,
};
use crate::mem::{ConstPtr, ConstVoidPtr, MutPtr, SafeRead};
use crate::objc::nil;

const kAudioUnitType_Output: u32 = fourcc(b"auou");
const kAudioUnitSubType_RemoteIO: u32 = fourcc(b"rioc");
const kAudioUnitManufacturer_Apple: u32 = fourcc(b"appl");

#[derive(Default)]
pub struct State {
    pub audio_component_instances:
        HashMap<AudioComponentInstance, AudioComponentInstanceHostObject>,
}
impl State {
    pub fn get(framework_state: &mut crate::frameworks::State) -> &mut Self {
        &mut framework_state.audio_toolbox.audio_components
    }
}

#[derive(Clone)]
pub struct AudioComponentInstanceHostObject {
    pub started: bool,
    pub global_stream_format: AudioStreamBasicDescription,
    pub output_stream_format: Option<AudioStreamBasicDescription>,
    pub render_callback: Option<AURenderCallbackStruct>,
    pub last_render_time: Option<Instant>,
    pub al_source: Option<ALuint>,
}
impl Default for AudioComponentInstanceHostObject {
    fn default() -> Self {
        AudioComponentInstanceHostObject {
            started: false,
            global_stream_format: AudioStreamBasicDescription {
                sample_rate: 44100.0,
                format_id: kAudioFormatLinearPCM,
                format_flags: kAudioFormatFlagIsFloat
                    | kAudioFormatFlagIsSignedInteger
                    | kAudioFormatFlagIsPacked
                    | kAudioFormatFlagIsAlignedHigh,
                bytes_per_packet: 4,
                frames_per_packet: 1,
                bytes_per_frame: 4,
                channels_per_frame: 2,
                bits_per_channel: 32,
                _reserved: 0,
            },
            output_stream_format: None,
            render_callback: None,
            last_render_time: None,
            al_source: None,
        }
    }
}

#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct AURenderCallbackStruct {
    pub inputProc: AURenderCallback,
    pub inputProcRefCon: ConstVoidPtr,
}
unsafe impl SafeRead for AURenderCallbackStruct {}
impl std::fmt::Debug for AURenderCallbackStruct {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let &AURenderCallbackStruct {
            inputProc,
            inputProcRefCon,
        } = self;
        f.debug_struct("AURenderCallbackStruct")
            .field("inputProc", &inputProc)
            .field("inputProcRefCon", &inputProcRefCon)
            .finish()
    }
}

#[repr(C, packed)]
struct OpaqueAudioComponent {}
unsafe impl SafeRead for OpaqueAudioComponent {}

type AudioComponent = MutPtr<OpaqueAudioComponent>;

pub type AURenderCallback = GuestFunction;

#[repr(C, packed)]
pub struct OpaqueAudioComponentInstance {
    _pad: u8,
}
unsafe impl SafeRead for OpaqueAudioComponentInstance {}

pub type AudioComponentInstance = MutPtr<OpaqueAudioComponentInstance>;

#[repr(C, packed)]
struct AudioComponentDescription {
    componentType: u32,
    componentSubType: u32,
    componentManufacturer: u32,
    componentFlags: u32,
    componentFlagsMask: u32,
}
unsafe impl SafeRead for AudioComponentDescription {}

fn AudioComponentFindNext(
    env: &mut Environment,
    inComponent: AudioComponent,
    inDesc: ConstPtr<AudioComponentDescription>,
) -> AudioComponent {
    let audio_comp_descr = env.mem.read(inDesc);
    assert!(audio_comp_descr.componentType == kAudioUnitType_Output);
    assert!(audio_comp_descr.componentSubType == kAudioUnitSubType_RemoteIO);
    assert!(audio_comp_descr.componentManufacturer == kAudioUnitManufacturer_Apple);

    let out_component = nil.cast();
    log!(
        "TODO: AudioComponentFindNext({:?}, {:?}) -> {:?}",
        inComponent,
        inDesc,
        out_component
    );
    out_component
}

fn AudioComponentInstanceNew(
    env: &mut Environment,
    inComponent: AudioComponent,
    outInstance: MutPtr<AudioComponentInstance>,
) -> OSStatus {
    let host_object = AudioComponentInstanceHostObject::default();

    let guest_instance: AudioComponentInstance = env
        .mem
        .alloc_and_write(OpaqueAudioComponentInstance { _pad: 0 });
    State::get(&mut env.framework_state)
        .audio_component_instances
        .insert(guest_instance, host_object);

    env.mem.write(outInstance, guest_instance);

    let result = 0; // success
    log_dbg!(
        "AudioComponentInstanceNew({:?}, {:?}) -> {:?}",
        inComponent,
        outInstance,
        result
    );
    result
}

fn AudioComponentInstanceDispose(
    env: &mut Environment,
    inInstance: AudioComponentInstance,
) -> OSStatus {
    let result = if inInstance.is_null() {
        -50
    } else {
        State::get(&mut env.framework_state)
            .audio_component_instances
            .remove(&inInstance);
        env.mem.free(inInstance.cast());
        0
    };
    log_dbg!(
        "AudioComponentInstanceDispose({:?}) -> {:?}",
        inInstance,
        result
    );
    result
}

pub const FUNCTIONS: FunctionExports = &[
    export_c_func!(AudioComponentFindNext(_, _)),
    export_c_func!(AudioComponentInstanceNew(_, _)),
    export_c_func!(AudioComponentInstanceDispose(_)),
];