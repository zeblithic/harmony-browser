use harmony_content::book::{BookStore, MemoryBookStore};
use harmony_content::bundle::BundleBuilder;
use harmony_content::cid::ContentId;
use harmony_content::vine::{VineDescriptor, MIME_VINE_VIDEO};

pub struct VineFixture {
    pub bundle_cid: ContentId,
    pub video_cid: ContentId,
    pub video_data: Vec<u8>,
    pub descriptor: VineDescriptor,
}

pub fn demo_vines() -> Vec<VineFixture> {
    vec![
        build_vine_fixture([0xAA; 16], 1_700_000_000, Some("dancing cat"), &fake_video(1)),
        build_vine_fixture(
            [0xAA; 16],
            1_700_000_060,
            Some("sunset timelapse"),
            &fake_video(2),
        ),
        build_vine_fixture([0xBB; 16], 1_700_000_030, None, &fake_video(3)),
    ]
}

fn fake_video(seed: u8) -> Vec<u8> {
    // Placeholder bytes — not a real video. The frontend video element won't
    // actually play these, but the component behavior (loading, mute toggle,
    // trust gating) is exercisable.
    let mut data = vec![0u8; 1024];
    data[0..4].copy_from_slice(b"VINE");
    data[4] = seed;
    data
}

fn build_vine_fixture(
    creator: [u8; 16],
    timestamp: u64,
    title: Option<&str>,
    video_bytes: &[u8],
) -> VineFixture {
    let mut store = MemoryBookStore::new();
    let video_cid = store.insert(video_bytes).unwrap();

    let descriptor = VineDescriptor {
        creator_address: creator,
        created_at: timestamp,
        video_cid: video_cid.to_bytes(),
        title: title.map(Into::into),
        reshare_of: None,
    };
    let desc_bytes = descriptor.to_bytes();
    let desc_cid = store.insert(&desc_bytes).unwrap();

    let mut builder = BundleBuilder::new();
    builder.add(video_cid);
    builder.add(desc_cid);
    builder.with_metadata(video_bytes.len() as u64, 1, timestamp, MIME_VINE_VIDEO);
    let (_data, bundle_cid) = builder.build().unwrap();

    VineFixture {
        bundle_cid,
        video_cid,
        video_data: video_bytes.to_vec(),
        descriptor,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn demo_vines_build_successfully() {
        let fixtures = demo_vines();
        assert_eq!(fixtures.len(), 3);
        for f in &fixtures {
            assert_ne!(f.bundle_cid.to_bytes(), [0u8; 32]);
            assert!(!f.video_data.is_empty());
        }
    }

    #[test]
    fn demo_vines_have_unique_cids() {
        let fixtures = demo_vines();
        let cids: Vec<_> = fixtures.iter().map(|f| f.bundle_cid.to_bytes()).collect();
        for i in 0..cids.len() {
            for j in (i + 1)..cids.len() {
                assert_ne!(cids[i], cids[j], "fixtures {i} and {j} have same CID");
            }
        }
    }

    #[test]
    fn demo_vines_have_correct_creators() {
        let fixtures = demo_vines();
        assert_eq!(fixtures[0].descriptor.creator_address, [0xAA; 16]);
        assert_eq!(fixtures[1].descriptor.creator_address, [0xAA; 16]);
        assert_eq!(fixtures[2].descriptor.creator_address, [0xBB; 16]);
    }
}
