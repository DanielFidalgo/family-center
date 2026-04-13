import React from 'react';

interface Props {
  name: string;
  color: string;
  avatarUrl?: string;
  size?: number;
}

const Avatar: React.FC<Props> = ({ name, color, avatarUrl, size = 28 }) => {
  const initial = name.charAt(0).toUpperCase();
  const fontSize = Math.round(size * 0.44);

  if (avatarUrl) {
    return (
      <img
        src={avatarUrl}
        alt={name}
        style={{
          width: size,
          height: size,
          borderRadius: '50%',
          objectFit: 'cover',
          flexShrink: 0,
          border: `2px solid ${color}`,
        }}
      />
    );
  }

  return (
    <div
      style={{
        width: size,
        height: size,
        borderRadius: '50%',
        background: color,
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        flexShrink: 0,
        color: '#fff',
        fontFamily: 'var(--fc-font-display)',
        fontWeight: 700,
        fontSize,
        lineHeight: 1,
        textShadow: '0 1px 2px rgba(0,0,0,0.3)',
      }}
    >
      {initial}
    </div>
  );
};

export default Avatar;
