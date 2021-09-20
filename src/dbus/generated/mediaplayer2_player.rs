#![allow(dead_code)]
use arg::{RefArg, Variant};
use dbus;
use dbus::arg;
use dbus::tree;
use std::collections::HashMap;

#[allow(clippy::type_complexity)]
pub trait OrgMprisMediaPlayer2Player {
    type Err;
    fn next(&self) -> Result<(), Self::Err>;
    fn previous(&self) -> Result<(), Self::Err>;
    fn pause(&self) -> Result<(), Self::Err>;
    fn play_pause(&self) -> Result<(), Self::Err>;
    fn stop(&self) -> Result<(), Self::Err>;
    fn play(&self) -> Result<(), Self::Err>;
    fn seek(&self, offset: i64) -> Result<(), Self::Err>;
    fn set_position(&self, track_id: dbus::Path, position: i64) -> Result<(), Self::Err>;
    fn open_uri(&self, uri: &str) -> Result<(), Self::Err>;
    fn get_playback_status(&self) -> Result<String, Self::Err>;
    fn get_loop_status(&self) -> Result<String, Self::Err>;
    fn set_loop_status(&self, value: String) -> Result<(), Self::Err>;
    fn get_rate(&self) -> Result<f64, Self::Err>;
    fn set_rate(&self, value: f64) -> Result<(), Self::Err>;
    fn get_shuffle(&self) -> Result<bool, Self::Err>;
    fn set_shuffle(&self, value: bool) -> Result<(), Self::Err>;
    fn get_metadata(
        &self,
    ) -> Result<HashMap<String, Variant<Box<dyn RefArg + 'static>>>, Self::Err>;
    fn get_volume(&self) -> Result<f64, Self::Err>;
    fn set_volume(&self, value: f64) -> Result<(), Self::Err>;
    fn get_position(&self) -> Result<i64, Self::Err>;
    fn get_minimum_rate(&self) -> Result<f64, Self::Err>;
    fn get_maximum_rate(&self) -> Result<f64, Self::Err>;
    fn get_can_go_next(&self) -> Result<bool, Self::Err>;
    fn get_can_go_previous(&self) -> Result<bool, Self::Err>;
    fn get_can_play(&self) -> Result<bool, Self::Err>;
    fn get_can_pause(&self) -> Result<bool, Self::Err>;
    fn get_can_seek(&self) -> Result<bool, Self::Err>;
    fn get_can_control(&self) -> Result<bool, Self::Err>;
}

#[allow(clippy::type_complexity)]
impl<'a, C: ::std::ops::Deref<Target = dbus::Connection>> OrgMprisMediaPlayer2Player
    for dbus::ConnPath<'a, C>
{
    type Err = dbus::Error;

    fn next(&self) -> Result<(), Self::Err> {
        let mut m = self.method_call_with_args(
            &"org.mpris.MediaPlayer2.Player".into(),
            &"Next".into(),
            |_| {},
        )?;
        m.as_result()?;
        Ok(())
    }

    fn previous(&self) -> Result<(), Self::Err> {
        let mut m = self.method_call_with_args(
            &"org.mpris.MediaPlayer2.Player".into(),
            &"Previous".into(),
            |_| {},
        )?;
        m.as_result()?;
        Ok(())
    }

    fn pause(&self) -> Result<(), Self::Err> {
        let mut m = self.method_call_with_args(
            &"org.mpris.MediaPlayer2.Player".into(),
            &"Pause".into(),
            |_| {},
        )?;
        m.as_result()?;
        Ok(())
    }

    fn play_pause(&self) -> Result<(), Self::Err> {
        let mut m = self.method_call_with_args(
            &"org.mpris.MediaPlayer2.Player".into(),
            &"PlayPause".into(),
            |_| {},
        )?;
        m.as_result()?;
        Ok(())
    }

    fn stop(&self) -> Result<(), Self::Err> {
        let mut m = self.method_call_with_args(
            &"org.mpris.MediaPlayer2.Player".into(),
            &"Stop".into(),
            |_| {},
        )?;
        m.as_result()?;
        Ok(())
    }

    fn play(&self) -> Result<(), Self::Err> {
        let mut m = self.method_call_with_args(
            &"org.mpris.MediaPlayer2.Player".into(),
            &"Play".into(),
            |_| {},
        )?;
        m.as_result()?;
        Ok(())
    }

    fn seek(&self, offset: i64) -> Result<(), Self::Err> {
        let mut m = self.method_call_with_args(
            &"org.mpris.MediaPlayer2.Player".into(),
            &"Seek".into(),
            |msg| {
                let mut i = arg::IterAppend::new(msg);
                i.append(offset);
            },
        )?;
        m.as_result()?;
        Ok(())
    }

    fn set_position(&self, track_id: dbus::Path, position: i64) -> Result<(), Self::Err> {
        let mut m = self.method_call_with_args(
            &"org.mpris.MediaPlayer2.Player".into(),
            &"SetPosition".into(),
            |msg| {
                let mut i = arg::IterAppend::new(msg);
                i.append(track_id);
                i.append(position);
            },
        )?;
        m.as_result()?;
        Ok(())
    }

    fn open_uri(&self, uri: &str) -> Result<(), Self::Err> {
        let mut m = self.method_call_with_args(
            &"org.mpris.MediaPlayer2.Player".into(),
            &"OpenUri".into(),
            |msg| {
                let mut i = arg::IterAppend::new(msg);
                i.append(uri);
            },
        )?;
        m.as_result()?;
        Ok(())
    }

    fn get_playback_status(&self) -> Result<String, Self::Err> {
        <Self as dbus::stdintf::org_freedesktop_dbus::Properties>::get(
            self,
            "org.mpris.MediaPlayer2.Player",
            "PlaybackStatus",
        )
    }

    fn get_loop_status(&self) -> Result<String, Self::Err> {
        <Self as dbus::stdintf::org_freedesktop_dbus::Properties>::get(
            self,
            "org.mpris.MediaPlayer2.Player",
            "LoopStatus",
        )
    }

    fn get_rate(&self) -> Result<f64, Self::Err> {
        <Self as dbus::stdintf::org_freedesktop_dbus::Properties>::get(
            self,
            "org.mpris.MediaPlayer2.Player",
            "Rate",
        )
    }

    fn get_shuffle(&self) -> Result<bool, Self::Err> {
        <Self as dbus::stdintf::org_freedesktop_dbus::Properties>::get(
            self,
            "org.mpris.MediaPlayer2.Player",
            "Shuffle",
        )
    }

    fn get_metadata(
        &self,
    ) -> Result<HashMap<String, arg::Variant<Box<dyn arg::RefArg + 'static>>>, Self::Err> {
        <Self as dbus::stdintf::org_freedesktop_dbus::Properties>::get(
            self,
            "org.mpris.MediaPlayer2.Player",
            "Metadata",
        )
    }

    fn get_volume(&self) -> Result<f64, Self::Err> {
        <Self as dbus::stdintf::org_freedesktop_dbus::Properties>::get(
            self,
            "org.mpris.MediaPlayer2.Player",
            "Volume",
        )
    }

    fn get_position(&self) -> Result<i64, Self::Err> {
        <Self as dbus::stdintf::org_freedesktop_dbus::Properties>::get(
            self,
            "org.mpris.MediaPlayer2.Player",
            "Position",
        )
    }

    fn get_minimum_rate(&self) -> Result<f64, Self::Err> {
        <Self as dbus::stdintf::org_freedesktop_dbus::Properties>::get(
            self,
            "org.mpris.MediaPlayer2.Player",
            "MinimumRate",
        )
    }

    fn get_maximum_rate(&self) -> Result<f64, Self::Err> {
        <Self as dbus::stdintf::org_freedesktop_dbus::Properties>::get(
            self,
            "org.mpris.MediaPlayer2.Player",
            "MaximumRate",
        )
    }

    fn get_can_go_next(&self) -> Result<bool, Self::Err> {
        <Self as dbus::stdintf::org_freedesktop_dbus::Properties>::get(
            self,
            "org.mpris.MediaPlayer2.Player",
            "CanGoNext",
        )
    }

    fn get_can_go_previous(&self) -> Result<bool, Self::Err> {
        <Self as dbus::stdintf::org_freedesktop_dbus::Properties>::get(
            self,
            "org.mpris.MediaPlayer2.Player",
            "CanGoPrevious",
        )
    }

    fn get_can_play(&self) -> Result<bool, Self::Err> {
        <Self as dbus::stdintf::org_freedesktop_dbus::Properties>::get(
            self,
            "org.mpris.MediaPlayer2.Player",
            "CanPlay",
        )
    }

    fn get_can_pause(&self) -> Result<bool, Self::Err> {
        <Self as dbus::stdintf::org_freedesktop_dbus::Properties>::get(
            self,
            "org.mpris.MediaPlayer2.Player",
            "CanPause",
        )
    }

    fn get_can_seek(&self) -> Result<bool, Self::Err> {
        <Self as dbus::stdintf::org_freedesktop_dbus::Properties>::get(
            self,
            "org.mpris.MediaPlayer2.Player",
            "CanSeek",
        )
    }

    fn get_can_control(&self) -> Result<bool, Self::Err> {
        <Self as dbus::stdintf::org_freedesktop_dbus::Properties>::get(
            self,
            "org.mpris.MediaPlayer2.Player",
            "CanControl",
        )
    }

    fn set_loop_status(&self, value: String) -> Result<(), Self::Err> {
        <Self as dbus::stdintf::org_freedesktop_dbus::Properties>::set(
            self,
            "org.mpris.MediaPlayer2.Player",
            "LoopStatus",
            value,
        )
    }

    fn set_rate(&self, value: f64) -> Result<(), Self::Err> {
        <Self as dbus::stdintf::org_freedesktop_dbus::Properties>::set(
            self,
            "org.mpris.MediaPlayer2.Player",
            "Rate",
            value,
        )
    }

    fn set_shuffle(&self, value: bool) -> Result<(), Self::Err> {
        <Self as dbus::stdintf::org_freedesktop_dbus::Properties>::set(
            self,
            "org.mpris.MediaPlayer2.Player",
            "Shuffle",
            value,
        )
    }

    fn set_volume(&self, value: f64) -> Result<(), Self::Err> {
        <Self as dbus::stdintf::org_freedesktop_dbus::Properties>::set(
            self,
            "org.mpris.MediaPlayer2.Player",
            "Volume",
            value,
        )
    }
}

#[allow(clippy::too_many_lines)]
pub fn org_mpris_media_player2_player_server<F, T, D>(
    factory: &tree::Factory<tree::MTFn<D>, D>,
    data: D::Interface,
    f: F,
) -> tree::Interface<tree::MTFn<D>, D>
where
    D: tree::DataType,
    D::Method: Default,
    D::Property: Default,
    D::Signal: Default,
    T: OrgMprisMediaPlayer2Player<Err = tree::MethodErr>,
    F: 'static + for<'z> Fn(&'z tree::MethodInfo<tree::MTFn<D>, D>) -> &'z T,
{
    let interface = factory.interface("org.mpris.MediaPlayer2.Player", data);
    let f = ::std::sync::Arc::new(f);
    let fclone = f.clone();
    let handle = move |minfo: &tree::MethodInfo<tree::MTFn<D>, D>| {
        let d = fclone(minfo);
        d.next()?;
        let rm = minfo.msg.method_return();
        Ok(vec![rm])
    };
    let method = factory.method("Next", Default::default(), handle);
    let interface = interface.add_m(method);

    let fclone = f.clone();
    let handle2 = move |minfo: &tree::MethodInfo<tree::MTFn<D>, D>| {
        let d = fclone(minfo);
        d.previous()?;
        let rm = minfo.msg.method_return();
        Ok(vec![rm])
    };
    let method = factory.method("Previous", Default::default(), handle2);
    let interface = interface.add_m(method);

    let fclone = f.clone();
    let handle3 = move |minfo: &tree::MethodInfo<tree::MTFn<D>, D>| {
        let d = fclone(minfo);
        d.pause()?;
        let rm = minfo.msg.method_return();
        Ok(vec![rm])
    };
    let method = factory.method("Pause", Default::default(), handle3);
    let interface = interface.add_m(method);

    let fclone = f.clone();
    let handle4 = move |minfo: &tree::MethodInfo<tree::MTFn<D>, D>| {
        let d = fclone(minfo);
        d.play_pause()?;
        let rm = minfo.msg.method_return();
        Ok(vec![rm])
    };
    let method = factory.method("PlayPause", Default::default(), handle4);
    let interface = interface.add_m(method);

    let fclone = f.clone();
    let handle5 = move |minfo: &tree::MethodInfo<tree::MTFn<D>, D>| {
        let d = fclone(minfo);
        d.stop()?;
        let rm = minfo.msg.method_return();
        Ok(vec![rm])
    };
    let method = factory.method("Stop", Default::default(), handle5);
    let interface = interface.add_m(method);

    let fclone = f.clone();
    let handle6 = move |minfo: &tree::MethodInfo<tree::MTFn<D>, D>| {
        let d = fclone(minfo);
        d.play()?;
        let rm = minfo.msg.method_return();
        Ok(vec![rm])
    };
    let method = factory.method("Play", Default::default(), handle6);
    let interface = interface.add_m(method);

    let fclone = f.clone();
    let handle7 = move |minfo: &tree::MethodInfo<tree::MTFn<D>, D>| {
        let mut i = minfo.msg.iter_init();
        let offset: i64 = i.read()?;
        let d = fclone(minfo);
        d.seek(offset)?;
        let rm = minfo.msg.method_return();
        Ok(vec![rm])
    };
    let method = factory.method("Seek", Default::default(), handle7);
    let method = method.in_arg(("Offset", "x"));
    let interface = interface.add_m(method);

    let fclone = f.clone();
    let handle8 = move |minfo: &tree::MethodInfo<tree::MTFn<D>, D>| {
        let mut i = minfo.msg.iter_init();
        let track_id: dbus::Path = i.read()?;
        let position: i64 = i.read()?;
        let destination = fclone(minfo);
        destination.set_position(track_id, position)?;
        let rm = minfo.msg.method_return();
        Ok(vec![rm])
    };
    let method = factory.method("SetPosition", Default::default(), handle8);
    let method = method.in_arg(("TrackId", "o"));
    let method = method.in_arg(("Position", "x"));
    let interface = interface.add_m(method);

    let fclone = f.clone();
    let handle9 = move |minfo: &tree::MethodInfo<tree::MTFn<D>, D>| {
        let mut interface = minfo.msg.iter_init();
        let uri: &str = interface.read()?;
        let destination = fclone(minfo);
        destination.open_uri(uri)?;
        let rm = minfo.msg.method_return();
        Ok(vec![rm])
    };
    let method = factory.method("OpenUri", Default::default(), handle9);
    let method = method.in_arg(("Uri", "s"));
    let interface = interface.add_m(method);

    let property = factory.property::<&str, _>("PlaybackStatus", Default::default());
    let property = property.access(tree::Access::Read);
    let fclone = f.clone();
    let property = property.on_get(move |a, pinfo| {
        let minfo = pinfo.to_method_info();
        let destination = fclone(&minfo);
        a.append(destination.get_playback_status()?);
        Ok(())
    });
    let interface = interface.add_p(property);

    let p = factory.property::<&str, _>("LoopStatus", Default::default());
    let p = p.access(tree::Access::ReadWrite);
    let fclone = f.clone();
    let p = p.on_get(move |a, pinfo| {
        let minfo = pinfo.to_method_info();
        let destination = fclone(&minfo);
        a.append(destination.get_loop_status()?);
        Ok(())
    });
    let fclone = f.clone();
    let p = p.on_set(move |iter, pinfo| {
        let minfo = pinfo.to_method_info();
        let d = fclone(&minfo);
        d.set_loop_status(iter.read()?)?;
        Ok(())
    });
    let interface = interface.add_p(p);

    let p = factory.property::<f64, _>("Rate", Default::default());
    let p = p.access(tree::Access::ReadWrite);
    let fclone = f.clone();
    let p = p.on_get(move |a, pinfo| {
        let minfo = pinfo.to_method_info();
        let d = fclone(&minfo);
        a.append(d.get_rate()?);
        Ok(())
    });
    let fclone = f.clone();
    let p = p.on_set(move |iter, pinfo| {
        let minfo = pinfo.to_method_info();
        let d = fclone(&minfo);
        d.set_rate(iter.read()?)?;
        Ok(())
    });
    let interface = interface.add_p(p);

    let p = factory.property::<bool, _>("Shuffle", Default::default());
    let p = p.access(tree::Access::ReadWrite);
    let fclone = f.clone();
    let p = p.on_get(move |a, pinfo| {
        let minfo = pinfo.to_method_info();
        let d = fclone(&minfo);
        a.append(d.get_shuffle()?);
        Ok(())
    });
    let fclone = f.clone();
    let p = p.on_set(move |iter, pinfo| {
        let minfo = pinfo.to_method_info();
        let d = fclone(&minfo);
        d.set_shuffle(iter.read()?)?;
        Ok(())
    });
    let interface = interface.add_p(p);

    let p = factory
        .property::<::std::collections::HashMap<&str, arg::Variant<Box<dyn arg::RefArg>>>, _>(
            "Metadata",
            Default::default(),
        );
    let p = p.access(tree::Access::Read);
    let fclone = f.clone();
    let p = p.on_get(move |a, pinfo| {
        let minfo = pinfo.to_method_info();
        let d = fclone(&minfo);
        a.append(d.get_metadata()?);
        Ok(())
    });
    let interface = interface.add_p(p);

    let p = factory.property::<f64, _>("Volume", Default::default());
    let p = p.access(tree::Access::ReadWrite);
    let fclone = f.clone();
    let p = p.on_get(move |a, pinfo| {
        let minfo = pinfo.to_method_info();
        let d = fclone(&minfo);
        a.append(d.get_volume()?);
        Ok(())
    });
    let fclone = f.clone();
    let p = p.on_set(move |iter, pinfo| {
        let minfo = pinfo.to_method_info();
        let d = fclone(&minfo);
        d.set_volume(iter.read()?)?;
        Ok(())
    });
    let interface = interface.add_p(p);

    let p = factory.property::<i64, _>("Position", Default::default());
    let p = p.access(tree::Access::Read);
    let fclone = f.clone();
    let p = p.on_get(move |a, pinfo| {
        let minfo = pinfo.to_method_info();
        let d = fclone(&minfo);
        a.append(d.get_position()?);
        Ok(())
    });
    let i = interface.add_p(p);

    let p = factory.property::<f64, _>("MinimumRate", Default::default());
    let p = p.access(tree::Access::Read);
    let fclone = f.clone();
    let p = p.on_get(move |a, pinfo| {
        let minfo = pinfo.to_method_info();
        let d = fclone(&minfo);
        a.append(d.get_minimum_rate()?);
        Ok(())
    });
    let i = i.add_p(p);

    let p = factory.property::<f64, _>("MaximumRate", Default::default());
    let p = p.access(tree::Access::Read);
    let fclone = f.clone();
    let p = p.on_get(move |a, pinfo| {
        let minfo = pinfo.to_method_info();
        let d = fclone(&minfo);
        a.append(d.get_maximum_rate()?);
        Ok(())
    });
    let i = i.add_p(p);

    let p = factory.property::<bool, _>("CanGoNext", Default::default());
    let p = p.access(tree::Access::Read);
    let fclone = f.clone();
    let p = p.on_get(move |a, pinfo| {
        let minfo = pinfo.to_method_info();
        let d = fclone(&minfo);
        a.append(d.get_can_go_next()?);
        Ok(())
    });
    let i = i.add_p(p);

    let p = factory.property::<bool, _>("CanGoPrevious", Default::default());
    let p = p.access(tree::Access::Read);
    let fclone = f.clone();
    let p = p.on_get(move |a, pinfo| {
        let minfo = pinfo.to_method_info();
        let d = fclone(&minfo);
        a.append(d.get_can_go_previous()?);
        Ok(())
    });
    let i = i.add_p(p);

    let p = factory.property::<bool, _>("CanPlay", Default::default());
    let p = p.access(tree::Access::Read);
    let fclone = f.clone();
    let p = p.on_get(move |a, pinfo| {
        let minfo = pinfo.to_method_info();
        let d = fclone(&minfo);
        a.append(d.get_can_play()?);
        Ok(())
    });
    let i = i.add_p(p);

    let p = factory.property::<bool, _>("CanPause", Default::default());
    let p = p.access(tree::Access::Read);
    let fclone = f.clone();
    let p = p.on_get(move |a, pinfo| {
        let minfo = pinfo.to_method_info();
        let d = fclone(&minfo);
        a.append(d.get_can_pause()?);
        Ok(())
    });
    let i = i.add_p(p);

    let p = factory.property::<bool, _>("CanSeek", Default::default());
    let p = p.access(tree::Access::Read);
    let fclone = f.clone();
    let p = p.on_get(move |a, pinfo| {
        let minfo = pinfo.to_method_info();
        let d = fclone(&minfo);
        a.append(d.get_can_seek()?);
        Ok(())
    });
    let i = i.add_p(p);

    let p = factory.property::<bool, _>("CanControl", Default::default());
    let p = p.access(tree::Access::Read);
    let fclone = f;
    let p = p.on_get(move |a, pinfo| {
        let minfo = pinfo.to_method_info();
        let d = fclone(&minfo);
        a.append(d.get_can_control()?);
        Ok(())
    });
    let i = i.add_p(p);
    let s = factory.signal("Seeked", Default::default());
    let s = s.arg(("Position", "x"));
    i.add_s(s)
}

#[derive(Debug, Default)]
pub struct OrgMprisMediaPlayer2PlayerSeeked {
    pub position: i64,
}

impl dbus::SignalArgs for OrgMprisMediaPlayer2PlayerSeeked {
    const NAME: &'static str = "Seeked";
    const INTERFACE: &'static str = "org.mpris.MediaPlayer2.Player";
    fn append(&self, i: &mut arg::IterAppend) {
        (&self.position as &dyn arg::RefArg).append(i);
    }
    fn get(&mut self, i: &mut arg::Iter) -> Result<(), arg::TypeMismatchError> {
        self.position = i.read()?;
        Ok(())
    }
}

#[derive(Debug, Default)]
pub struct OrgFreedesktopDBusPropertiesPropertiesChanged {
    pub interface_name: String,
    pub changed_properties:
        ::std::collections::HashMap<String, arg::Variant<Box<dyn arg::RefArg + 'static>>>,
    pub invalidated_properties: Vec<String>,
}

impl dbus::SignalArgs for OrgFreedesktopDBusPropertiesPropertiesChanged {
    const NAME: &'static str = "PropertiesChanged";
    const INTERFACE: &'static str = "org.freedesktop.DBus.Properties";
    fn append(&self, i: &mut arg::IterAppend) {
        (&self.interface_name as &dyn arg::RefArg).append(i);
        (&self.changed_properties as &dyn arg::RefArg).append(i);
        (&self.invalidated_properties as &dyn arg::RefArg).append(i);
    }
    fn get(&mut self, i: &mut arg::Iter) -> Result<(), arg::TypeMismatchError> {
        self.interface_name = i.read()?;
        self.changed_properties = i.read()?;
        self.invalidated_properties = i.read()?;
        Ok(())
    }
}
