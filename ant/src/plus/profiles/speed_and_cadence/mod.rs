use packed_struct::derive::PackedStruct;
use derive_new::new;

const DEVICE_TYPE: u8 = 121;

#[derive(PackedStruct, new, PartialEq, Copy, Clone, Debug)]
#[packed_struct(endian = "lsb")]
pub struct SpeedAndCadence {
    /// Time of the last valid bike cadence event (1/1024 sec)
    pub cadence_event_time: u16,

    /// Total number of pedal revolutions
    pub cadence_revolution_count: u16,

    /// Time of the last valid bike speed event (1/1024 sec)
    pub speed_event_time: u16,

    /// Total number of wheel revolutions
    pub speed_revolution_count: u16,
}

impl SpeedAndCadence {
    /// Calculates the average cadence (rpm)
    pub fn cadence(a: SpeedAndCadence, b: SpeedAndCadence) -> Option<f32> {
        let time_delta = b.cadence_event_time.wrapping_sub(a.cadence_event_time);
        if time_delta == 0 {
            return None
        }
        let rev_delta = b.cadence_revolution_count.wrapping_sub(a.cadence_revolution_count);
        Some((rev_delta as f32) * 1024.0 * 60.0 / (time_delta as f32))
    }

    /// Calculates the number of wheel revolutions
    pub fn wheel_revolutions(a: SpeedAndCadence, b: SpeedAndCadence) -> Option<u16> {
        let time_delta = b.speed_event_time.wrapping_sub(a.speed_event_time);
        if time_delta == 0 {
            return None
        }
        Some(b.speed_revolution_count.wrapping_sub(a.speed_revolution_count))
    }

    /// Calculates the distance (m) covered between two messages
    pub fn distance(a: SpeedAndCadence, b: SpeedAndCadence, circumference: f32) -> Option<f32> {
        if let Some(revs) = Self::wheel_revolutions(a, b) {
            return Some(revs as f32 * circumference)
        }
        None
    }

    /// Calculates average speed in revolutions per sec (useful when circumference is not known)
    pub fn speed_revs_per_sec(a: SpeedAndCadence, b: SpeedAndCadence) -> Option<f32> {
        if let Some(revs) = Self::wheel_revolutions(a, b) {
            let time_delta = b.speed_event_time.wrapping_sub(a.speed_event_time);
            return Some(revs as f32 * 1024.0 / time_delta as f32)
        }
        None
    }

    /// Calculates average speed (m/s)
    pub fn speed(a: SpeedAndCadence, b: SpeedAndCadence, circumference: f32) -> Option<f32> {
        if let Some(speed) = Self::speed_revs_per_sec(a, b) {
            return Some(speed * circumference)
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use packed_struct::PackedStruct;
    use super::*;

    #[test]
    fn unpack() {
        let raw = [0x09, 0x91, 0xd5, 0x08, 0xd7, 0x90, 0x42, 0x1b];
        let foo = SpeedAndCadence::unpack(&raw).unwrap();
        assert_eq!(foo.cadence_event_time, 37129);
        assert_eq!(foo.cadence_revolution_count, 2261);
        assert_eq!(foo.speed_event_time, 37079);
        assert_eq!(foo.speed_revolution_count, 6978);
    }

    #[test]
    fn cadence() {
        // If the timer hasn't changed we should return None
        let a = SpeedAndCadence::new(0, 0, 0, 0);
        assert_eq!(SpeedAndCadence::cadence(a, a), None);

        let a = SpeedAndCadence::new(0, 0, 0, 0);
        let b = SpeedAndCadence::new(1024, 1, 0, 0);
        assert!((SpeedAndCadence::cadence(a, b).unwrap() - 60.0).abs() <= f32::EPSILON);

        // test counter roll-over
        let a = SpeedAndCadence::new(u16::MAX, u16::MAX, 0, 0);
        let b = SpeedAndCadence::new(1023, 0, 0, 0);
        assert!((SpeedAndCadence::cadence(a, b).unwrap() - 60.0).abs() <= f32::EPSILON);
    }

    #[test]
    fn speed() {
        // If the timer hasn't changed we should return None
        let a = SpeedAndCadence::new(0, 0, 0, 0);
        assert_eq!(SpeedAndCadence::speed(a, a, 1.0), None);

        let a = SpeedAndCadence::new(0, 0, 0, 0);
        let b = SpeedAndCadence::new(0, 0, 1024, 1);
        assert!((SpeedAndCadence::speed(a, b, 1.0).unwrap() - 1.0).abs() <= f32::EPSILON);

        // test counter roll-over
        let a = SpeedAndCadence::new( 0, 0, u16::MAX, u16::MAX);
        let b = SpeedAndCadence::new( 0, 0, 1023, 0);
        assert!((SpeedAndCadence::speed(a, b, 1.0).unwrap() - 1.0).abs() <= f32::EPSILON);
    }
}