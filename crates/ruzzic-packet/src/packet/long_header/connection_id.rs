use crate::packet::{PacketTransformError, TransformToLongHeaderError};

#[cfg_attr(test, derive(Debug, PartialEq))]
pub struct ConnectionId<'a> {
    pub id: &'a [u8],
    // Position just after reading id.
    pub position_after: usize,
}

pub fn connection_id(buf: &[u8], position: usize) -> Result<ConnectionId, PacketTransformError> {
    let length = *buf
        .get(position)
        .ok_or(PacketTransformError::UnexpectedEnd(position))? as usize;
    let position = position + 1;
    if length > 20 {
        return Err(TransformToLongHeaderError::ConnectionIdIsInvalidLength(
            length,
        ))
        .map_err(Into::into);
    }

    let id = buf
        .get(position..position + length)
        .ok_or(PacketTransformError::UnexpectedEnd(position))?;
    let position_after = position + length;
    Ok(ConnectionId { id, position_after })
}

#[cfg(test)]
mod tests {
    use super::{connection_id, ConnectionId};

    use crate::packet::{PacketTransformError, TransformToLongHeaderError};

    #[test]
    fn connection_id_length_20() -> Result<(), PacketTransformError> {
        let input = [&[20u8][..], &[0u8; 20][..]].concat();
        let cid = connection_id(&input, 0)?;
        assert_eq!(
            cid,
            ConnectionId {
                id: &[0u8; 20],
                position_after: 21,
            }
        );
        Ok(())
    }

    #[test]
    fn connection_id_length_21() -> Result<(), PacketTransformError> {
        let input = [&[21u8][..], &[0u8; 21][..]].concat();
        let result = connection_id(&input, 0);
        assert_eq!(
            result,
            Err(PacketTransformError::TransformToLong(
                TransformToLongHeaderError::ConnectionIdIsInvalidLength(21)
            ))
        );
        Ok(())
    }
}
