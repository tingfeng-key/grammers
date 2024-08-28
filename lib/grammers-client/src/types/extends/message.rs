use crate::types;

pub fn fmt_entities_new(msg: &types::Message) -> Vec<super::entity::Entity> {
    super::entity::Entity::_from_message(msg.text(), msg.fmt_entities().map(|x| x.clone()))
}

pub fn parse_usernames_from_entities(msg: &types::Message) -> Vec<String> {
    // println!("origin entities:{:#?}", self.fmt_entities());
    let entities: Vec<String> = fmt_entities_new(msg)
        .iter()
        .filter_map(|x| x.username())
        .collect::<std::collections::HashSet<String>>()
        .into_iter()
        .collect();
    // println!("new entities:{:#?}", entities);
    entities
}
