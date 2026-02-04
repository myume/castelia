export enum StreamStatus {
  Offline = "offline",
  Unpublished = "unpublished",
  Published = "published",
}

export interface Broadcast {
  channel_name: string;
  title: string;
  start_time?: string;
  status: StreamStatus;
  private: boolean;
}
