use godot::classes::AnimatedSprite2D;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AnimationResumeData {
    pub frame: i32,
    pub progress: f32,
}

pub trait AnimatedSpriteExt {
    fn stop_with_resume_data(&mut self) -> AnimationResumeData;
    fn resume(&mut self, resume_data: AnimationResumeData);
}

impl AnimatedSpriteExt for AnimatedSprite2D {
    fn stop_with_resume_data(&mut self) -> AnimationResumeData {
        let data = AnimationResumeData {
            frame: self.get_frame(),
            progress: self.get_frame_progress(),
        };
        self.stop();
        data
    }

    fn resume(&mut self, resume_data: AnimationResumeData) {
        self.play();
        self.set_frame_and_progress(resume_data.frame, resume_data.progress);
    }
}
