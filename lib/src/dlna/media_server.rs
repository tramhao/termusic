use std::{error::Error, fmt::Write};
use rupnp::Device;
use rupnp::ssdp::URN;
use serde::de::IntoDeserializer;
use crate::dlna::models::{DlnaDevice, MediaContainer, MediaItem};

#[derive(Debug)]
pub struct MediaServerController {
    pub device: DlnaDevice,
}

impl MediaServerController {
    pub  fn new(device: DlnaDevice) -> Self {
        Self {
            device,
        }
    }
    
    async fn find_content_directory(&self) -> Result<rupnp::Service, Box<dyn Error>> {
        let urn = URN::service("schemas-upnp-org", "ContentDirectory", 1);
        let url = self.device.uri.parse().unwrap();
        let device = Device::from_url(url).await?;
        if let Some(service) = device.find_service(&urn) {
            return Ok(service.clone())
        }
        Err("No ContentDirectory service found".into())
    }
    
    pub async fn browse_directory(&self, object_id: &str) -> Result<(), Box<dyn Error>> {
        let service = self.find_content_directory().await?;
        
        let mut args = String::new();
        write!(args, "<ObjectID>{}</ObjectID>", object_id)?;
        write!(args, "<BrowseFlag>BrowseDirectChildren</BrowseFlag>")?;
        write!(args, "<Filter>*</Filter>")?;
        write!(args, "<StartingIndex>1</StartingIndex>")?;
        write!(args, "<RequestedCount>0</RequestedCount>")?;
        
        let url = self.device.uri.parse().unwrap();
        let hash_result = service.action(&url, "Browser", &args).await?;
        let result = &hash_result["Result"];
        
        self.parse_browse_result(&result).await
    }
    
    async fn parse_browse_result(&self, result_xml: &str) -> Result<(), Box<dyn Error>> {
        let mut item_count = 0;
        
        if let Ok(didl) = xmltree::Element::parse(result_xml.as_bytes()) {
            for child in didl.children.iter() {
                if let xmltree::XMLNode::Element(item_elem) = child {
                    if item_elem.name == "container" {
                        if let Some(media_container) = self.parse_media_container(item_elem) {
                            println!("Container {}", media_container.name);
                            if media_container.name == "Music" || media_container.name == "All Music" {
                                Box::pin(self.browse_directory(media_container.id.as_str())).await?;

                            }
                            //Box::pin(self.browse_directory(media_container.id.as_str())).await?;
                        }
                    }
                    if item_elem.name == "item" {
                        if let Some(media_item) = self.parse_media_item(item_elem) {
                            item_count += 1;
                            println!("{}: {:?} - {} ({:?}) [{:?}] - {}", media_item.id, media_item.artist, media_item.title, media_item.album, media_item.duration, media_item.url);
                        }
                    }
                }
            }
        }
        println!("Found {} elements", item_count);
        Ok(())
    }

    fn parse_media_container(&self, item_elem: &xmltree::Element) -> Option<MediaContainer> {
        let id = item_elem.attributes.get("id")?.clone();

        let name = item_elem.get_child("title")
            .and_then(|e| e.children.first())
            .and_then(|n| if let xmltree::XMLNode::Text(text) = n { Some(text.clone()) } else { None })
            .unwrap_or_else(|| "Unknown title".to_string());

        Some( MediaContainer{ id,  name } )
    }

    fn parse_media_item(&self, item_elem: &xmltree::Element) -> Option<MediaItem> {
        let id = item_elem.attributes.get("id")?.clone();

        let title = item_elem.get_child("title")
            .and_then(|e| e.children.first())
            .and_then(|n| if let xmltree::XMLNode::Text(text) = n { Some(text.clone()) } else { None })
            .unwrap_or_else(|| "Unknown title".to_string());

        let res = item_elem.get_child("res")
            .and_then(|e| e.children.first())
            .and_then(|n| if let xmltree::XMLNode::Text(text) = n { Some(text.clone()) } else { None })?;

        let duration = item_elem.get_child("res")
            .and_then(|e| e.attributes.get("duration"))
            .cloned();

        let artist = item_elem.get_child("artist")
            .and_then(|e| e.children.first())
            .and_then(|n| if let xmltree::XMLNode::Text(text) = n { Some(text.clone()) } else { None });

        let album = item_elem.get_child("album")
            .and_then(|e| e.children.first())
            .and_then(|n| if let xmltree::XMLNode::Text(text) = n { Some(text.clone()) } else { None });

        Some( MediaItem {
            id,
            title,
            url: res,
            duration,
            artist,
            album,
        })
    }

}