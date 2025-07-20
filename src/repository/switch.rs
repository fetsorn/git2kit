use crate::{Repository, Result};

pub fn switch(repository: &Repository, reference: &git2::Reference) -> Result<()> {
    repository.repo.checkout_tree(
        &reference.peel(git2::ObjectType::Tree)?,
        Some(git2::build::CheckoutBuilder::new().safe()),
    )?;

    repository
        .repo
        .set_head(reference.name().expect("ref name is invalid utf-8"))?;

    Ok(())
}
