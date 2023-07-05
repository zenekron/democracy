use serenity::builder::{
    CreateComponents, CreateEmbed, CreateInteractionResponseData, EditMessage,
};

pub enum MessageRenderer<'a, 'b> {
    CreateInteractionResponseData(&'a mut CreateInteractionResponseData<'b>),
    EditMessage(&'a mut EditMessage<'b>),
}

impl<'a, 'b> MessageRenderer<'a, 'b> {
    pub fn set_components(&mut self, components: CreateComponents) -> &mut Self {
        match self {
            MessageRenderer::CreateInteractionResponseData(message) => {
                message.set_components(components);
            }
            MessageRenderer::EditMessage(message) => {
                message.set_components(components);
            }
        }

        self
    }

    pub fn set_embeds(&mut self, embeds: impl IntoIterator<Item = CreateEmbed>) -> &mut Self {
        match self {
            MessageRenderer::CreateInteractionResponseData(message) => {
                message.set_embeds(embeds);
            }
            MessageRenderer::EditMessage(message) => {
                message.set_embeds(embeds.into_iter().collect());
            }
        }

        self
    }
}

impl<'a, 'b> From<&'a mut CreateInteractionResponseData<'b>> for MessageRenderer<'a, 'b> {
    fn from(value: &'a mut CreateInteractionResponseData<'b>) -> Self {
        Self::CreateInteractionResponseData(value)
    }
}

impl<'a, 'b> From<&'a mut EditMessage<'b>> for MessageRenderer<'a, 'b> {
    fn from(value: &'a mut EditMessage<'b>) -> Self {
        Self::EditMessage(value)
    }
}
