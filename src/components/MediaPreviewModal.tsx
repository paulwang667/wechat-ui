import React from 'react';
import { X, Download, ZoomIn, ZoomOut } from 'lucide-react';

interface MediaPreviewModalProps {
  imageUrl: string | null;
  onClose: () => void;
  isDarkMode: boolean;
}

export const MediaPreviewModal: React.FC<MediaPreviewModalProps> = ({
  imageUrl,
  onClose,
}) => {
  if (!imageUrl) return null;

  return (
    <div className="fixed inset-0 bg-black/85 backdrop-blur-md flex items-center justify-center p-4 z-50 select-none animate-fadeIn">
      {/* Top action bar */}
      <div className="absolute top-4 right-4 flex items-center space-x-2">
        <a
          href={imageUrl}
          target="_blank"
          rel="noreferrer"
          download="wechat_image_preview.jpg"
          className="p-2 rounded-full bg-white/10 hover:bg-white/20 text-white transition-colors"
          title="Download image"
        >
          <Download className="w-4 h-4" />
        </a>
        <button
          onClick={onClose}
          className="p-2 rounded-full bg-white/10 hover:bg-white/20 text-white transition-colors"
          title="Close preview"
        >
          <X className="w-4 h-4" />
        </button>
      </div>

      <div className="max-w-4xl max-h-[85vh] flex items-center justify-center">
        <img
          src={imageUrl}
          alt="WeChat Media Preview"
          className="max-w-full max-h-[85vh] object-contain rounded-lg shadow-2xl"
        />
      </div>
    </div>
  );
};
