use filesystem;
use soundcloud;
use time;

const BLOCK_SIZE: u64 = 1024;

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "soundcloud error: {}", _0)]
    SoundCloudError(soundcloud::Error),
}

impl filesystem::Error for Error {
    fn errno(&self) -> i32 {
        match self {
            Error::SoundCloudError(_) => libc::EIO,
        }
    }
}

impl From<soundcloud::Error> for Error {
    fn from(err: soundcloud::Error) -> Error {
        Error::SoundCloudError(err)
    }
}

#[derive(Clone, Debug)]
pub enum Entry<'a> {
    User(soundcloud::User<'a>),
    UserFavorites(soundcloud::User<'a>),
    UserFollowing(soundcloud::User<'a>),
    Track(soundcloud::Track<'a>),
}

impl<'a> filesystem::Node<'a> for Entry<'a> {
    type Error = Error;

    fn file_attributes(&self, ino: u64) -> fuse::FileAttr {
        let now = time::now().to_timespec();
        match self {
            Entry::User(_user) => fuse::FileAttr {
                ino,
                size: 0,
                blocks: 1,
                atime: now,
                mtime: now,
                ctime: now,
                crtime: now,
                kind: fuse::FileType::Directory,
                perm: 0o555,
                nlink: 1,
                uid: 0,
                gid: 0,
                rdev: 1,
                flags: 0,
            },
            Entry::UserFavorites(_) => fuse::FileAttr {
                ino,
                size: 0,
                blocks: 1,
                atime: now,
                mtime: now,
                ctime: now,
                crtime: now,
                kind: fuse::FileType::Directory,
                perm: 0o555,
                nlink: 1,
                uid: 0,
                gid: 0,
                rdev: 1,
                flags: 0,
            },
            Entry::UserFollowing(_) => fuse::FileAttr {
                ino,
                size: 0,
                blocks: 1,
                atime: now,
                mtime: now,
                ctime: now,
                crtime: now,
                kind: fuse::FileType::Directory,
                perm: 0o555,
                nlink: 1,
                uid: 0,
                gid: 0,
                rdev: 1,
                flags: 0,
            },
            Entry::Track(track) => {
                let perm = if track.audio_accessible() { 0o444 } else { 0 };
                fuse::FileAttr {
                    ino,
                    size: track.original_content_size,
                    blocks: track.original_content_size / BLOCK_SIZE + 1,
                    atime: now,
                    mtime: now,
                    ctime: now,
                    crtime: now,
                    kind: fuse::FileType::RegularFile,
                    perm,
                    nlink: 1,
                    uid: 0,
                    gid: 0,
                    rdev: 1,
                    flags: 0,
                }
            }
        }
    }

    fn open_ro(&self) -> Result<Box<filesystem::ReadSeek + 'a>, Error> {
        match self {
            Entry::Track(track) => Ok(Box::new(track.audio()?)),
            _ => unreachable!("only tracks can be opened for reading"),
        }
    }

    fn children(&self) -> Result<Vec<(String, Entry<'a>)>, Error> {
        match self {
            Entry::User(user) => {
                let mut children = Vec::new();
                children.push(("favorites".to_string(), Entry::UserFavorites(user.clone())));
                if user.primary_email_confirmed.is_some() {
                    // Only add the following directory for the logged in user to prevent recursing
                    // too deeply.
                    children.push(("following".to_string(), Entry::UserFollowing(user.clone())));
                }
                children.extend(
                    user.tracks()?
                        .into_iter()
                        .map(|track| map_track_to_child(track)),
                );
                Ok(children)
            }
            Entry::UserFavorites(user) => {
                let children: Vec<_> = user
                    .favorites()?
                    .into_iter()
                    .map(|track| map_track_to_child(track))
                    .collect();
                Ok(children)
            }
            Entry::UserFollowing(user) => {
                let children: Vec<_> = user
                    .following()?
                    .into_iter()
                    .map(|user| (user.permalink.clone(), Entry::User(user)))
                    .collect();
                Ok(children)
            }
            Entry::Track(_) => unreachable!("tracks do not have child files"),
        }
    }
}

fn map_track_to_child(track: soundcloud::Track) -> (String, Entry) {
    let title = track
        .title
        .replace(|c: char| !c.is_alphanumeric() && !c.is_whitespace(), "")
        .replace("  ", " ")
        .replace(|c: char| c.is_whitespace(), "_");
    let name = format!("{}_{}.mp3", title, track.id);
    (name, Entry::Track(track))
}
