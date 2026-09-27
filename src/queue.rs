use crate::library::Track;

#[derive(Default)]
pub struct Queue {
    pub tracks: Vec<Track>,
    pub current: Option<usize>,
}

impl Queue {
    pub fn current(&self) -> Option<&Track> {
        self.current.and_then(|index| self.tracks.get(index))
    }

    pub fn replace(&mut self, tracks: Vec<Track>, start: usize) {
        self.tracks = tracks;
        self.current = if self.tracks.is_empty() {
            None
        } else {
            Some(start.min(self.tracks.len() - 1))
        };
    }

    pub fn append(&mut self, track: Track) {
        self.tracks.push(track);
        if self.current.is_none() {
            self.current = Some(0);
        }
    }

    pub fn queue_next(&mut self, track: Track) {
        match self.current {
            Some(i) => self.tracks.insert(i + 1, track),
            None => self.append(track),
        }
    }

    pub fn next(&mut self) -> Option<&Track> {
        let next = self.current.map(|i| i + 1).unwrap_or(0);

        if next < self.tracks.len() {
            self.current = Some(next);
            self.current()
        } else {
            self.current = None; // end
            None
        }
    }

    pub fn prev(&mut self) -> Option<&Track> {
        let i = self.current?;
        if i > 0 {
            self.current = Some(i - 1);
        }
        self.current()
    }

    pub fn remove(&mut self, index: usize) {
        if index >= self.tracks.len() {
            return;
        }

        self.tracks.remove(index);
        if let Some(current) = self.current {
            if self.tracks.is_empty() {
                self.current = None
            } else if index < current {
                self.current = Some(current - 1);
            } else if current >= self.tracks.len() {
                self.current = Some(self.tracks.len() - 1)
            }
        }
    }

    pub fn jump_to(&mut self, index: usize) {
        self.current = Some(index);
    }

    pub fn upcoming(&self) -> &[Track] {
        match self.current {
            Some(i) => &self.tracks[(i + 1).min(self.tracks.len())..],
            None => &self.tracks,
        }
    }
}
